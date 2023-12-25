use anyhow::{anyhow, Result};
use std::{
    path::{Path, PathBuf},
    process::{exit, Command},
};

use crate::get_pathbuf;

// Function to get the top-level directory of the Git repository
pub fn get_git_top_level_dir() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;

    if output.status.success() {
        get_pathbuf(String::from_utf8_lossy(&output.stdout).trim())
    } else {
        Err(anyhow!("Error: Failed to get Git top-level directory"))
    }
}

#[test]
fn main() {
    match get_git_top_level_dir() {
        Ok(git_top_level_dir) => {
            println!("Git top-level directory: {:?}", git_top_level_dir);
            // Now you can use git_top_level_dir as needed in your program
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
    }
}
