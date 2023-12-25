use crate::*;
use anyhow::{anyhow, Ok, Result};
use arboard::Clipboard;
use std::{
    fs::{create_dir_all, metadata, read_to_string, remove_dir_all, remove_file, write},
    io::{stdin, stdout, Write},
    path::{Path, PathBuf},
    process::{exit, Command, Stdio},
};

#[derive(Debug, Clone, Default)]
pub struct GitInit {
    pub host: String,
    pub name: String,
    pub email: String,
    pub label: String,
    pub ssh_dir: Option<PathBuf>,
    pub private_key: Option<PathBuf>,
    pub public_key: Option<PathBuf>,
    pub config_file: Option<PathBuf>,
    pub config_content: String,
    pub regenerate_key_pair: bool,
}

/*
impl fmt::Display for GitInit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Git Information:")?;
        writeln!(f, "Label: {}", self.label)?;
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Host: {}", self.host)?;
        writeln!(f, "Email: {}", self.email)?;

        if let Some(ssh_dir) = &self.ssh_dir {
            writeln!(f, "SSH Dir: {}", ssh_dir.display())?;
        }

        if let Some(private_key) = &self.private_key {
            writeln!(f, "Private Key: {}", private_key.display())?;
        }

        if let Some(public_key) = &self.public_key {
            writeln!(f, "Public Key: {}", public_key.display())?;
        }

        if let Some(known_hosts_file) = &self.known_hosts_file {
            writeln!(f, "Known Hosts File: {}", known_hosts_file.display())?;
        }

        if let Some(config_file) = &self.config_file {
            writeln!(f, "Config File: {}", config_file.display())?;
        }

        writeln!(f, "Config Content:\n{}", self.config_content)?;

        Ok(())
    }
}
*/

impl GitInit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn regenerate(mut self) -> Self {
        self.regenerate_key_pair = true;
        self
    }

    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = email.into();
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_ssh_dir<P: AsRef<Path>>(mut self, path_or_name: P) -> Self {
        self.ssh_dir = Some(path_or_name.as_ref().into());
        self
    }

    pub fn with_config<P: AsRef<Path>>(mut self, path_or_name: P) -> Self {
        self.config_file = Some(path_or_name.as_ref().into());
        self
    }

    pub fn with_key<P: AsRef<Path>>(mut self, path_or_name: P) -> Self {
        self.private_key = Some(path_or_name.as_ref().into());
        self
    }

    pub fn print(&mut self) {
        GitConfig::Local.list();
        println!("{:#?}", self);
    }

    fn update(&mut self) -> Result<Self> {
        //> Sys Info
        let username = whoami::username();
        let hostname = whoami::devicename();
        let distro = whoami::distro();

        //> Label, Name, and Host
        macro_rules! set_if_empty {
            ($field:expr, $value:expr) => {
                if $field.is_empty() {
                    $field = $value;
                }
            };
        }

        set_if_empty!(
            self.label,
            format!("{}@{} on {}", username, hostname, distro)
        );
        set_if_empty!(self.name, username.to_lowercase());
        set_if_empty!(self.host, hostname.to_lowercase());

        //> SSH Dir
        let ssh_dir = get_abs_path(
            self.ssh_dir.as_deref(),
            get_ssh_home()?.as_path(),
            [""],
            "ssh_dir",
        )?;
        self.ssh_dir = Some(ssh_dir.clone());

        //> Private Key
        let private_key = get_abs_path(
            self.private_key.as_deref(),
            &ssh_dir,
            [&self.host, &self.name],
            "private_key",
        )?;
        self.private_key = Some(private_key.clone());

        //> Public Key
        let public_key = private_key.with_extension("pub");
        self.public_key = Some(public_key.clone());

        //> Config Path
        let config_file = get_abs_path(
            self.config_file.as_deref(),
            &ssh_dir,
            ["config"],
            "config_file",
        )?;
        self.config_file = Some(config_file.clone());

        //> Config Content
        let config_content = format!(
            "Host {}\n\tUser {}\n\tHostName {}\n\tIdentityFile {}\n",
            &self.host,
            &self.name,
            &self.host,
            &private_key.display(),
        );
        self.config_content = config_content.clone();

        //> Git Info
        Ok(self.clone())
    }

    fn execute(&mut self) -> Result<()> {
        if let Err(err) = self.update() {
            eprintln!("Error: {}", err);
            exit(1);
        }

        let private_key = self.private_key.as_ref().unwrap();
        let public_key = self.public_key.as_ref().unwrap();
        let label = self.label.as_str();

        // generate_ssh_key_pair(
        //     &self.private_key,
        //     &self.public_key,
        //     &self.label,
        //     self.regenerate_key_pair,
        // )?;
        // update_ssh_config(&self.config_file, &self.config_content)?;
        // // activate_via_pull(&self.host, &self.name, &private_key)?;
        GitConfig::Local.set_value("user.name", &self.name);
        GitConfig::Local.set_value("user.email", &self.email);

        Ok(())
    }
}

fn generate_ssh_key_pair(
    private_key: &Path,
    public_key: &Path,
    label: &str,
    reset: bool,
) -> Result<()> {
    //> Check if both keys exist and reset is not selected
    if private_key.exists() && public_key.exists() && !reset {
        //> println!("SSH keys already exist. Skipping key generation.");
        return Ok(());
    }

    //> If reset is selected or any of the keys is missing, remove both keys
    if reset || !private_key.exists() || !public_key.exists() {
        if private_key.exists() {
            remove_file(private_key)?;
        }

        if public_key.exists() {
            remove_file(public_key)?;
        }
    }

    //> Create the parent directory if necessary
    if let Some(parent) = private_key.parent() {
        if !parent.exists() {
            create_dir_all(parent)?;
        }
    }

    //> Generate the SSH key pair
    let cmd_keygen = Command::new("ssh-keygen")
        .arg("-t")
        .arg("ed25519") // Algorithm
        .arg("-a")
        .arg("100") // Rounds
        .arg("-f")
        .arg(private_key)
        .arg("-C")
        .arg(label) // Label
        .stdin(Stdio::inherit()) // Inherit stdin for password prompt
        .stdout(Stdio::inherit()) // Inherit stdout for password confirmation
        .stderr(Stdio::inherit()) // Inherit stderr for error messages
        .output()?;

    if cmd_keygen.status.success() {
        //> Send the fingerprint to the clipboard
        let mut clipboard = Clipboard::new().unwrap();
        clipboard
            .set_text(read_to_string(public_key)?.trim().to_string())
            .unwrap();

        println!(
            "SSH keys generated successfully and the public key has been copied to the clipboard."
        );
        Ok(())
    } else {
        Err(anyhow!(
            "SSH Keygen Error |> {} |> {}",
            cmd_keygen.status,
            String::from_utf8_lossy(&cmd_keygen.stderr)
        ))
    }
}

fn update_ssh_config(config_file: &Path, config_content: &str) -> Result<()> {
    //> Create the parent directory if necessary
    if let Some(parent) = config_file.parent() {
        if !parent.exists() {
            create_dir_all(parent)?;
        }
    }

    //> Check if the config file exists
    if config_file.exists() {
        //> If the file exists, read the current content
        let current_content = read_to_string(config_file)?;

        //> Check if the content is already present
        if current_content.contains(config_content) {
            return Ok(());
        }

        //> Append the new content to the existing content
        let new_content = format!("{}\n{}", current_content, config_content);

        //> Write the combined content to the config file
        write(config_file, new_content)?;
    } else {
        //> If the file doesn't exist, create it with the specified content
        write(config_file, config_content)?;
        //TODO: Replace stdout with log
        println!("Config file created successfully with the specified content.");
    }

    Ok(())
}

fn activate_via_pull(hostname: &str, name: &str, private_key: &Path) -> Result<()> {
    //> Determine the target directory
    let repository = format!("git@{}:{}/{}.git", hostname, name, name);
    let target_dir = private_key
        .parent()
        .ok_or(anyhow!("Invalid private key path"))?;
    let target_repo = target_dir.join(format!("{}-repo", name));
    let target_repo_str = target_repo.to_string_lossy().to_string();

    //> Check if the directory exists
    if metadata(&target_repo).is_ok() {
        remove_dir_all(&target_repo)?;
    } else {
        let cmd = Command::new("git")
            .arg("clone")
            .arg(repository)
            .arg(&target_repo_str)
            .output()?;

        if !cmd.status.success() {
            return Err(anyhow!("Failed to clone the repository."));
        }
    }

    //> Remove the existing repository
    if metadata(&target_repo).is_ok() {
        remove_dir_all(&target_repo)?;
    }

    Ok(())
}

fn initialize_git_in_current_directory() -> Result<()> {
    let cmd_git_init = Command::new("git")
        .arg("init")
        .arg(".")
        .stdin(Stdio::inherit()) // Inherit stdin for password prompt
        .stdout(Stdio::inherit()) // Inherit stdout for password confirmation
        .stderr(Stdio::inherit()) // Inherit stderr for error messages
        .output()?;

    if cmd_git_init.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "Failed to initialize Git in the current directory."
        ))
    }
}

pub fn git_info_test() {
    let mut git_info = GitInit::new();

    git_info = git_info.with_name("craole-cc");
    git_info = git_info.with_host("github.com");
    git_info = git_info.with_email("craole@tuta.io");

    // git_info = git_info.with_key("pop");
    // git_info = git_info.with_key("pop/lol");
    // git_info = git_info.with_key("C:/pop/lol");
    git_info.print();
}
