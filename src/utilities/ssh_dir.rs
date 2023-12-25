use anyhow::{Context, Result};
use std::{env, path::PathBuf};

pub fn get_ssh_home() -> Result<PathBuf> {
    let error_msg = "Failed to determine SSH home directory";
    let home_dir = dirs::home_dir();

    home_dir.map_or_else(
        || {
            let home_var = env::var("HOME").or_else(|_| env::var("USERPROFILE"));

            home_var
                .map_or_else(
                    |_| Err(anyhow::anyhow!(error_msg)),
                    |home| Ok(PathBuf::from(home).join(".ssh")),
                )
                .context(error_msg)
        },
        |path: PathBuf| Ok(path.join(".ssh")),
    )
}
