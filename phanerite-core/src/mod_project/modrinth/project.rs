use crate::download::Downloader;
use crate::error::{Error, Result};
use crate::mod_project::features::display::{ModDisplay, Rgb};
use crate::mod_project::features::filter::{FilterCriteria, ModFilter};
use crate::mod_project::modrinth::serde::{DetailProject, SearchProject, Version};
use crate::mod_project::modrinth::MODRINTH_API;
use crate::mod_project::ModProject;
use async_lock::OnceCell;
use chrono::{DateTime, FixedOffset};
use std::fmt::Display;
use url::Url;

pub struct Project<'repo, D: Downloader> {
    downloader: &'repo D,

    common: SearchProject,
    detail: OnceCell<DetailProject>,
    versions: OnceCell<Vec<Version>>,
    filter: OnceCell<FilterCriteria>,
}

impl<D: Downloader> Project<'_, D> {
    pub(super) fn new(common: SearchProject, downloader: &D) -> Project<'_, D> {
        Project {
            downloader,
            common,
            detail: Default::default(),
            versions: Default::default(),
            filter: Default::default(),
        }
    }
    pub(super) async fn detail(&self) -> Result<&DetailProject> {
        self.detail
            .get_or_try_init(async || {
                let mut url = MODRINTH_API.clone();
                url.path_segments_mut()
                    .expect("cannot-be-a-base URL")
                    .extend(["project", &self.common.project_id]);
                let body = self.downloader.fetch(url, None).await?;
                let json = serde_json::from_slice::<DetailProject>(&body)?;
                Ok(json)
            })
            .await
    }
}

impl<D: Downloader> ModProject for Project<'_, D> {
    type ModVersion = Version;
    async fn versions(&self) -> Result<impl Iterator<Item = &Self::ModVersion>> {
        Ok(self
            .versions
            .get_or_try_init(async || {
                let mut url = MODRINTH_API.clone();
                url.path_segments_mut()
                    .expect("cannot-be-a-base URL")
                    .extend(["project", &self.common.project_id, "version"]);
                let body = self.downloader.fetch(url, None).await?;
                let json = serde_json::from_slice::<Vec<Version>>(&body)?;
                Ok::<Vec<Version>, Error>(json)
            })
            .await?
            .iter())
    }
}

impl<D: Downloader> ModDisplay for Project<'_, D> {
    async fn title(&self) -> impl Display + '_ {
        &self.common.title
    }
    async fn description(&self) -> impl Display + '_ {
        &self.common.description
    }
    async fn body(&self) -> impl Display + '_ {
        self.detail()
            .await
            .map(|t| t.body.as_str())
            .unwrap_or("Failed to load. Please try again.")
    }
    async fn author(&self) -> impl Display + '_ {
        &self.common.author
    }
    async fn created_time(&self) -> &DateTime<FixedOffset> {
        &self.common.date_created
    }
    async fn updated_time(&self) -> &DateTime<FixedOffset> {
        &self.common.date_modified
    }
    async fn license(&self) -> impl Iterator<Item = impl Display + '_> {
        self.common.license.split_ascii_whitespace()
    }
    async fn categories(&self) -> impl Iterator<Item = impl Display + '_> {
        self.common.display_categories.iter()
    }
    async fn icon(&self) -> Option<Url> {
        Some(self.common.icon_url.clone())
    }
    async fn color(&self) -> Option<Rgb> {
        Some(Rgb {
            r: ((self.common.color >> 16) & 0xff) as u8,
            g: ((self.common.color >> 8) & 0xff) as u8,
            b: (self.common.color & 0xff) as u8,
        })
    }
    async fn downloads(&self) -> Option<usize> {
        Some(self.common.downloads)
    }
    async fn follows(&self) -> Option<usize> {
        Some(self.common.follows)
    }
    async fn gallery(&self) -> Vec<Url> {
        self.common.gallery.clone()
    }
}

impl<D: Downloader> ModFilter for Project<'_, D> {
    async fn filter_criteria(&self) -> Result<&FilterCriteria> {
        self.filter
            .get_or_try_init(async || {
                let detail = self.detail().await?;
                Ok(FilterCriteria {
                    project_type: Some(self.common.project_type.into()),
                    mods_loader: detail
                        .loaders
                        .iter()
                        .map(|x| x.parse().expect("unreachable"))
                        .collect(),
                    game_version: self.common.versions.clone(),
                    loader_version: detail.versions.clone(),
                })
            })
            .await
    }
}
