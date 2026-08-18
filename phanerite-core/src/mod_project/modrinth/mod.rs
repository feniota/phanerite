pub mod project;
pub mod repo;
pub mod serde;
pub mod version;

pub use repo::Repository;
use std::sync::LazyLock;
use url::Url;

static MODRINTH_API: LazyLock<Url> = LazyLock::new(|| "https://api.modrinth.com/".parse().unwrap());
