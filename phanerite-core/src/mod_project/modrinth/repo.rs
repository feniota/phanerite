use crate::download::Downloader;
use crate::error::Result;
use crate::mod_project::ModsRepository;
use crate::mod_project::modrinth::MODRINTH_API;
use crate::mod_project::modrinth::project::Project;
use crate::mod_project::modrinth::serde::SearchProject;
use futures::{Stream, StreamExt};
use serde::Deserialize;
use std::sync::LazyLock;

const PROJECT_PER_PAGE: usize = 20;
static PROJECT_PER_PAGE_STR: LazyLock<String> = LazyLock::new(|| PROJECT_PER_PAGE.to_string());

pub struct Repository<'downloader, D: Downloader> {
    downloader: &'downloader D,
}

async fn fetch_page<'repo, D: Downloader + 'repo>(
    downloader: &'repo D,
    keyword: &str,
    offset: usize,
) -> Result<Option<Vec<Project<'repo, D>>>> {
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
        .append_pair("offset", format!("{offset}").as_str());
    let body = downloader.fetch(url, None).await?;
    let base = serde_json::from_slice::<SearchResponse>(&body)?;

    if offset >= base.total_hits || base.hits.is_empty() {
        return Ok(None);
    }

    let projects = base
        .hits
        .into_iter()
        .map(|common| Project::new(common, downloader))
        .collect();
    Ok(Some(projects))
}

impl<'repo, D: Downloader> ModsRepository for Repository<'repo, D> {
    const NAME: &'static str = "Modrinth";
    type ModType = Project<'repo, D>;
    fn search(&self, keyword: &str) -> impl Stream<Item = Result<Self::ModType>> {
        futures::stream::unfold(0usize, async move |offset| {
            match fetch_page(self.downloader, keyword, offset).await {
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
