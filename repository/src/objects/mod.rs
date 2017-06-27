mod file_objects;
mod git_objects;

use errors::*;
use git;
use self::file_objects::*;
use self::git_objects::*;
pub use sha256::Checksum;
use std::path::{Path, PathBuf};
use url::Url;

/// Configuration file for objects backends.
pub struct ObjectsConfig {
    /// Root path when checking out local repositories.
    pub repos: Option<PathBuf>,
}

pub trait Objects {
    /// Put the given object into the database.
    /// This will cause the object denoted by the given checksum to be uploaded to the objects
    /// store.
    fn put_object(&self, checksum: &Checksum, source: &Path) -> Result<()>;

    /// Get a path to the object with the given checksum.
    /// This might cause the object to be downloaded if it's not already present in the local
    /// filesystem.
    fn get_object(&self, checksum: &Checksum) -> Result<Option<PathBuf>>;
}

pub fn objects_from_file(url: &Url) -> Result<Box<Objects>> {
    let path = Path::new(url.path());

    if !path.is_dir() {
        return Err(format!("no such directory: {}", path.display()).into());
    }

    Ok(Box::new(FileObjects::new(&path)))
}

pub fn objects_from_git<'a, I>(config: ObjectsConfig,
                               scheme: I,
                               url: &'a Url)
                               -> Result<Box<Objects>>
    where I: IntoIterator<Item = &'a str>
{
    let mut scheme = scheme.into_iter();

    let sub_scheme = scheme.next()
        .ok_or_else(|| {
            format!("invalid scheme ({}), expected git+file, git+https, or git+ssh",
                    url.scheme())
        })?;

    let repos = config.repos.ok_or_else(|| "repos: not specified")?;

    let git_repo = git::setup_git_repo(&repos, sub_scheme, url)?;

    git_repo.fetch()?;

    let file_objects = FileObjects::new(git_repo.path());
    let objects = GitObjects::new(git_repo, file_objects);

    Ok(Box::new(objects))
}

pub fn objects_from_url(config: ObjectsConfig, url: &Url) -> Result<Box<Objects>> {
    let mut scheme = url.scheme().split("+");
    let first = scheme.next().ok_or_else(|| format!("invalid scheme: {}", url))?;

    match first {
        "file" => objects_from_file(url),
        "git" => objects_from_git(config, scheme, url),
        scheme => Err(format!("unsupported scheme ({}): {}", scheme, url).into()),
    }
}
