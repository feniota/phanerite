use crate::download::Downloader;
use crate::error::{Error, Result};
use crate::mod_project::ModsRepository;
use crate::mod_project::features::filter::{FilterCriteria, ModsRepositoryFilterExt};
use crate::mod_project::modrinth::project::{ExtendedProject, Project, SearchProject};
use crate::mod_project::modrinth::version::{DependencyType, Version};
use futures::Stream;
use futures_lite::stream::StreamExt;
use serde::Deserialize;
use std::sync::LazyLock;
use url::Url;

static MODRINTH_API: LazyLock<Url> = LazyLock::new(|| "https://api.modrinth.com/".parse().unwrap());

const PROJECT_PER_PAGE: usize = 20;
static PROJECT_PER_PAGE_STR: LazyLock<String> = LazyLock::new(|| PROJECT_PER_PAGE.to_string());

pub struct Repository<'downloader, D: Downloader> {
    downloader: &'downloader D,
}

impl<D: Downloader> Repository<'_, D> {
    fn search_inner(&self, keyword: &str, facets: &str) -> impl Stream<Item = Result<Project>> {
        async fn fetch_page(
            downloader: &impl Downloader,
            keyword: &str,
            offset: usize,
            facets: &str,
        ) -> Result<Option<Vec<Project>>> {
            #[derive(Deserialize)]
            struct SearchResponse {
                hits: Vec<SearchProject>,
                // offset: usize,
                // limit: usize,
                total_hits: usize,
            }

            let mut url = MODRINTH_API.clone();
            url.path_segments_mut()
                .expect("cannot-be-a-base URL")
                .push("search");
            url.query_pairs_mut()
                .append_pair("query", keyword)
                .append_pair("limit", &PROJECT_PER_PAGE_STR)
                .append_pair("offset", format!("{offset}").as_str())
                .append_pair("facets", facets);
            let body = downloader.fetch(url, None).await?;
            let base = serde_json::from_slice::<SearchResponse>(&body)?;

            if offset >= base.total_hits || base.hits.is_empty() {
                return Ok(None);
            }

            let ids = base
                .hits
                .iter()
                .map(|x| x.project_id.clone())
                .collect::<Vec<_>>();

            let mut url = MODRINTH_API.clone();
            url.path_segments_mut()
                .expect("cannot-be-a-base URL")
                .push("projects");
            url.query_pairs_mut()
                .append_pair("ids", &serde_json::to_string(&ids)?);
            let body = downloader.fetch(url, None).await?;
            let ext = serde_json::from_slice::<Vec<ExtendedProject>>(&body)?;

            let projects = base
                .hits
                .into_iter()
                .zip(ext)
                .map(|(search_project, extended_project)| Project {
                    search_project,
                    extended_project,
                    filter_criteria: Default::default(),
                })
                .collect();
            Ok(Some(projects))
        }

        futures::stream::unfold(0usize, async move |offset| {
            match fetch_page(self.downloader, keyword, offset, facets).await {
                Ok(Some(projects)) => {
                    let next_offset = offset + PROJECT_PER_PAGE;
                    Some((
                        projects.into_iter().map(Ok).collect::<Vec<_>>(),
                        next_offset,
                    ))
                }
                Ok(None) => None,
                Err(error) => Some((vec![Err(error)], usize::MAX)),
            }
        })
        .flat_map(futures::stream::iter)
    }
}

impl<D: Downloader> ModsRepository for Repository<'_, D> {
    const NAME: &'static str = "Modrinth";
    type ModType = Project;
    type ModVersion = Version;
    fn search(&self, keyword: &str) -> impl Stream<Item = Result<Self::ModType>> {
        self.search_inner(keyword, "")
    }

    async fn versions(
        &self,
        project: &Self::ModType,
    ) -> Result<impl Iterator<Item = Self::ModVersion>> {
        let mut url = MODRINTH_API.clone();
        url.path_segments_mut()
            .expect("cannot-be-a-base URL")
            .extend(["project", &project.search_project.project_id, "version"]);

        let body = self.downloader.fetch(url, None).await?;
        let json = serde_json::from_slice::<Vec<Version>>(&body)?;
        Ok(json.into_iter())
    }

    async fn dependencies(
        &self,
        version: &Self::ModVersion,
    ) -> Result<impl Iterator<Item = Self::ModVersion>> {
        let res = futures::stream::iter(version.dependencies.iter())
            .filter(|x| x.dependency_type.eq(&DependencyType::Required))
            .map(async |x| {
                let mut url = MODRINTH_API.clone();
                if let Some(v) = &x.version_id {
                    url.path_segments_mut()
                        .expect("cannot-be-a-base URL")
                        .extend(["version", v]);

                    let body = self.downloader.fetch(url, None).await?;
                    let version = serde_json::from_slice::<Version>(&body)?;

                    Ok(vec![version])
                } else if let Some(v) = &x.project_id {
                    url.path_segments_mut()
                        .expect("cannot-be-a-base URL")
                        .extend(["project", v, "version"]);

                    let body = self.downloader.fetch(url, None).await?;
                    let versions = serde_json::from_slice::<Vec<Version>>(&body)?;

                    Ok(versions)
                } else {
                    Err(Error::other("No available dependency"))
                }
            });
        let res = futures::stream::StreamExt::buffer_unordered(res, 16)
            .try_collect::<Vec<Version>, Error, Vec<_>>()
            .await?;
        Ok(res.into_iter().flatten())
    }
}

pub struct FilteredRepository<'repo, R: ModsRepository> {
    repo: &'repo R,
    facets: String,
}

impl<'a, D: Downloader> ModsRepository for FilteredRepository<'_, Repository<'a, D>> {
    const NAME: &'static str = Repository::<D>::NAME;
    const ATTRIBUTION: &'static str = Repository::<D>::ATTRIBUTION;
    const NOTICE: &'static str = Repository::<D>::NOTICE;
    type ModType = <Repository<'a, D> as ModsRepository>::ModType;
    type ModVersion = <Repository<'a, D> as ModsRepository>::ModVersion;

    fn search(&self, keyword: &str) -> impl Stream<Item = Result<Self::ModType>> {
        self.repo.search_inner(keyword, &self.facets)
    }
    async fn versions(
        &self,
        project: &Self::ModType,
    ) -> Result<impl Iterator<Item = Self::ModVersion>> {
        self.repo.versions(project).await
    }
    async fn dependencies(
        &self,
        version: &Self::ModVersion,
    ) -> Result<impl Iterator<Item = Self::ModVersion>> {
        self.repo.dependencies(version).await
    }
}

impl<D: Downloader> ModsRepositoryFilterExt for Repository<'_, D> {
    fn filtered(&self, filter_criteria: FilterCriteria) -> impl ModsRepository {
        let mut facets = vec![];
        if let Some(v) = filter_criteria.project_type {
            facets.push(vec![format!("project_type:{v}")])
        }
        if !filter_criteria.game_version.is_empty() {
            facets.push(
                filter_criteria
                    .game_version
                    .iter()
                    .map(|x| format!("versions:{x}"))
                    .collect(),
            );
        }
        if !filter_criteria.mods_loader.is_empty() {
            facets.push(
                filter_criteria
                    .mods_loader
                    .iter()
                    .map(|x| format!("categories:{x}"))
                    .collect(),
            );
        }
        let facets = serde_json::to_string(&facets).expect("unexpected serialization error");

        FilteredRepository { repo: self, facets }
    }
}
