use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// Computes the absolute path based on the provided path or fallback components.
///
/// # Arguments
///
/// * `path` - An optional path that, if provided, will be used to generate the absolute path.
/// * `fallback_parent` - The fallback parent path to be used when no path is provided.
/// * `fallback_components` - The components to be joined when no path is provided.
/// * `context` - Context information for error messages.
///
/// # Returns
///
/// Returns a `Result<PathBuf>` representing the absolute path or an error if the path cannot be obtained.
///
/// # Examples
///
/// ```
/// let path = get_abs_path(Some(Path::new("/example")), Path::new("/fallback"), &[""], "example_context");
/// ```
pub fn get_abs_path<I, T>(
    path: Option<&Path>,
    fallback_parent: &Path,
    fallback_components: I,
    context: &str,
) -> Result<PathBuf>
where
    I: IntoIterator<Item = T>,
    T: AsRef<Path>,
{
    let result = path.map_or_else(
        || {
            let buffed_path: PathBuf = fallback_components
                .into_iter()
                .fold(PathBuf::new(), |acc, component| acc.join(component));
            fallback_parent.join(buffed_path)
        },
        |provided| {
            let buffed_path: PathBuf = provided.components().map(|c| c.as_os_str()).collect();
            if buffed_path.is_absolute() {
                buffed_path
            } else {
                fallback_parent.join(buffed_path)
            }
        },
    );

    if result.exists() {
        Ok(result)
    } else {
        Err(anyhow!(
            "Path not found: {}, Context: {}",
            result.display(),
            context
        ))
    }
}

//...USAGE
// let private_key = get_abs_path(
//     self.private_key.as_deref(),
//     &ssh_dir,
//     [&self.host, &self.name],
//     "private_key",
// )?;
// self.private_key = Some(private_key.clone());

pub fn get_pathbuf<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();

    if path.is_absolute() {
        let buffed_path = path
            .iter()
            .fold(PathBuf::new(), |acc, component| acc.join(component));
        Ok(buffed_path)
    } else {
        Err(anyhow!("Provided path is not absolute"))
    }
}
