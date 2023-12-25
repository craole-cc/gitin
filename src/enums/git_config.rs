use anyhow::{anyhow, Result};
use std::io::{stdin, stdout, Write};
use std::process::{Command, Stdio};
use crate::*;

pub enum GitConfig {
    Global,
    Local,
    System,
    Worktree,
}

impl GitConfig {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Local => "local",
            Self::System => "system",
            Self::Worktree => "worktree",
        }
    }

    pub fn list(&self) -> Result<String> {
        let title = "Git Config";
        let scope_option = format!("--{}", self.to_str());
        let cmd_git_config_list = Command::new("git")
            .arg("config")
            .arg(&scope_option)
            .arg("--list")
            .output()
            .unwrap_or_else(|_| panic!("Failed to `git config {} --list`", scope_option));

        if cmd_git_config_list.status.success() {
            Ok(format!(
                "===| {}: {} |===\n{}",
                title,
                self.to_str().to_uppercase(),
                String::from_utf8_lossy(&cmd_git_config_list.stdout)
            ))
        } else {
            Err(anyhow!(
                "{} Error |> {} |> {}",
                title,
                cmd_git_config_list.status,
                String::from_utf8_lossy(&cmd_git_config_list.stderr),
            ))
        }
    }

    pub fn get_value(&self, key: &str) -> Result<Option<String>> {
        let output = Command::new("git")
            .arg("config")
            .arg(format!("--{}", self.to_str()))
            .arg("--get")
            .arg(key)
            .output()?;

        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(value))
        } else if String::from_utf8_lossy(&output.stderr)
            .contains("can only be used inside a git repository")
        {
            // if permission_granted("Git repository not initialized. Do you want to initialize it?") {
            if permission_granted("Git repository not initialized. Do you want to initialize it?")
            {
                // TODO: initialize_git_in_current_directory()?;
                // Now try getting the config value again
                self.get_value(key)
            } else {
                // User chose not to initialize, return None
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn set_value(&self, key: &str, value: &str) -> Result<()> {
        let current_value = self.get_value(key)?;

        if let Some(current_value) = current_value.as_deref() {
            if current_value == value {
                return Ok(());
            } else if permission_granted(format!(
                "Update the {} config key '{}' from '{}' to '{}'?",
                self.to_str(),
                key,
                current_value,
                value
            )) {
            } else {
                return Ok(());
            }
        }

        let cmd_update_key = Command::new("git")
            .arg("config")
            .arg(format!("--{}", self.to_str()))
            .arg("--replace-all")
            .arg(key)
            .arg(value)
            .output()?;

        if cmd_update_key.status.success() {
            println!(
                "Updated the {} git config: {} = {}",
                self.to_str(),
                key,
                value
            );
            Ok(())
        } else {
            Err(anyhow!("Failed to update Git config for key: {}", key))
        }
    }
}

fn get_git_dir() -> Option<String> {
    let cmd_output = Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .output()
        .ok()?;

    if cmd_output.status.success() {
        Some(
            String::from_utf8_lossy(&cmd_output.stdout)
                .trim()
                .to_string(),
        )
    } else {
        None
    }
}

#[test]
fn test_get_value() {
    let key = "user.name";
    let result = GitConfig::Local.get_value(key);
    match result {
        Ok(Some(value)) => println!("{} = {}", &key, value),
        Ok(None) => println!("Value not found for key: {}", &key),
        Err(error) => eprintln!("{}", error),
    }

    let key = "user.name";
    let result = GitConfig::Local.get_value(key);
    printres_opt!(result)
}

#[test]
fn test_set_value() {
    GitConfig::Local.set_value("user.name", "craole-cc");
    GitConfig::Local.set_value("user.email", "craole@tuta.io");
    GitConfig::Local.set_value("remote.origin.url", "git@github.com:craole-cc/gitsy.git");
}

#[test]
fn test_list() {
    printres!(GitConfig::Local.list());
    printres!(GitConfig::Global.list());
    printres!(GitConfig::Worktree.list());
    printres!(GitConfig::System.list());
}
