pub mod core;
pub mod install;
pub mod helper;

use crate::ramp::helper::get_user_home;

use std::io;
use std::env::consts;
use std::path::Path;
use std::fs::{ File, OpenOptions };
use std::io::{ Write,BufReader, BufRead };

pub struct Paths {
    sdk_path: Option<String>,
    ndk_path: Option<String>,
    cargo_path: Option<String>,
    cargo_apk_path: Option<String>,
    zigbuild_path: Option<String>,
    rustup_path: Option<String>,
    homebrew_path: Option<String>,
    cmdline_tools_path: Option<String>,
    build_tools_path: Option<String>,
    sdkmanager_path: Option<String>,
    platform_tools_path: Option<String>,
    platforms_path: Option<String>,
    ndk_bundle_path: Option<String>,
    java_path: Option<String>,
    keystore_path: Option<String>,
}

pub struct Certs {
    macos: String
}

pub struct Session {
    os: String,
    home: String,
    projects_path: Option<String>,
    current_project: Option<String>,
    paths: Paths,
    certs: Certs,
    android_ndk_version: String,
    android_platform_version: String,
}

impl Session {
    fn new() -> io::Result<Self> {
        let os = consts::OS.to_string();
        let home = get_user_home()?;
        let projects_path = match os.as_str() {
            "linux" => {
                Some(format!("{}/ramp", home))
            }
            "macos" => {
                Some(format!("{}/ramp", home))
            },
            //unsupported OS
            _ => None,
        };
        let paths = Paths {
            sdk_path: None,
            ndk_path: None,
            cargo_path: None,
            cargo_apk_path: None,
            zigbuild_path: None,
            rustup_path: None,
            homebrew_path: None,
            cmdline_tools_path: None,
            build_tools_path: None,
            sdkmanager_path: None,
            platform_tools_path: None,
            platforms_path: None,
            ndk_bundle_path: None,
            java_path: None,
            keystore_path: None,
        };
        let certs = Certs{
            macos: "Apple Development".to_string(),
        };
        Ok(Session {
            os,
            home,
            projects_path,
            current_project: None,
            paths,
            certs,
            android_ndk_version: "26.1.10909125".to_string(),
            android_platform_version: "31".to_string(),
        })
    }

    fn update_current_project(&mut self, name: String) -> io::Result<()> {
        let new_path = format!(
            "{}/{}",
            self.projects_path.as_ref().unwrap_or(&String::new()),
            name
        );
        //check that the requested project exists at the specificed path
        if !Path::new(&new_path).exists() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to load project, project not found",
            ));
        }
        //check the requested project for compatibility with ramp
        if Path::new(&format!("{}/.ramp", &new_path)).exists() {
            self.current_project = Some(name);
            return Ok(());
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to load project, not compatible with ramp",
            ));
        }
    }

    // Method to update a path in the Paths struct and .config file
    fn set_path(&mut self, path_name: &str, file_path: String) -> io::Result<()> {
        // Update the Paths struct
        println!("Updating config path {} to {}", path_name, file_path);
        match path_name {
            "sdk_path" => self.paths.sdk_path = Some(file_path.clone()),
            "ndk_path" => self.paths.ndk_path = Some(file_path.clone()),
            "cargo_path" => self.paths.cargo_path = Some(file_path.clone()),
            "cargo_apk_path" => self.paths.cargo_apk_path = Some(file_path.clone()),
            "zigbuild_path" => self.paths.zigbuild_path = Some(file_path.clone()),
            "rustup_path" => self.paths.rustup_path = Some(file_path.clone()),
            "homebrew_path" => self.paths.homebrew_path = Some(file_path.clone()),
            "cmdline_tools_path" => self.paths.cmdline_tools_path = Some(file_path.clone()),
            "build_tools_path" => self.paths.build_tools_path = Some(file_path.clone()),
            "sdkmanager_path" => self.paths.sdkmanager_path = Some(file_path.clone()),
            "platform_tools_path" => self.paths.platform_tools_path = Some(file_path.clone()),
            "platforms_path" => self.paths.platforms_path = Some(file_path.clone()),
            "ndk_bundle_path" => self.paths.ndk_bundle_path = Some(file_path.clone()),
            "java_path" => self.paths.java_path = Some(file_path.clone()),
            "keystore_path" => self.paths.keystore_path = Some(file_path.clone()),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unknown path name")),
        }

        // Read existing config file
        let config_path = format!("{}/.ramp", self.home);
        let mut config_lines = Vec::new();
        let mut found = false;

        if Path::new(&config_path).exists() {
            let file = File::open(&config_path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                if line.starts_with(&format!("{}=", path_name)) {
                    config_lines.push(format!("{}={}", path_name, file_path));
                    found = true;
                } else {
                    config_lines.push(line);
                }
            }
        }

        // If the path wasn't found, append it
        if !found {
            config_lines.push(format!("{}={}", path_name, file_path));
        }

        // Write updated config back to file
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&config_path)?;
        for line in config_lines {
            writeln!(file, "{}", line)?;
        }

        println!("Successfully updated path");

        Ok(())
    }

    // Method to read .config and update Paths struct
    fn get_all_paths(&mut self) -> io::Result<()> {
        let config_path = format!("{}/.ramp", self.home);

        if !Path::new(&config_path).exists() {
            return Ok(()); // No config file yet, keep default paths
        }

        let file = File::open(&config_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "sdk_path" => self.paths.sdk_path = Some(value.trim().to_string()),
                    "ndk_path" => self.paths.ndk_path = Some(value.trim().to_string()),
                    "cargo_path" => self.paths.cargo_path = Some(value.trim().to_string()),
                    "cargo_apk_path" => self.paths.cargo_apk_path = Some(value.trim().to_string()),
                    "zigbuild_path" => self.paths.zigbuild_path = Some(value.trim().to_string()),
                    "rustup_path" => self.paths.rustup_path = Some(value.trim().to_string()),
                    "homebrew_path" => self.paths.homebrew_path = Some(value.trim().to_string()),
                    "cmdline_tools_path" => self.paths.cmdline_tools_path = Some(value.trim().to_string()),
                    "build_tools_path" => self.paths.build_tools_path = Some(value.trim().to_string()),
                    "sdkmanager_path" => self.paths.sdkmanager_path = Some(value.trim().to_string()),
                    "platform_tools_path" => self.paths.platform_tools_path = Some(value.trim().to_string()),
                    "platforms_path" => self.paths.platforms_path = Some(value.trim().to_string()),
                    "ndk_bundle_path" => self.paths.ndk_bundle_path = Some(value.trim().to_string()),
                    "java_path" => self.paths.java_path = Some(value.trim().to_string()),
                    "keystore_path" => self.paths.keystore_path = Some(value.trim().to_string()),
                    _ => (), // Ignore unknown keys
                }
            }
        }

        Ok(())
    }

    fn get_path(&mut self, key: &str) -> io::Result<String>{
        match key {
            "sdk_path" => Ok(self.paths.sdk_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "sdk_path not set"))?
                .to_string()),
            "ndk_path" => Ok(self.paths.ndk_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "ndk_path not set"))?
                .to_string()),
            "cargo_path" => Ok(self.paths.cargo_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cargo_path not set"))?
                .to_string()),
            "cargo_apk_path" => Ok(self.paths.cargo_apk_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cargo_apk_path not set"))?
                .to_string()),
            "zigbuild_path" => Ok(self.paths.zigbuild_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "zigbuild_path not set"))?
                .to_string()),
            "rustup_path" => Ok(self.paths.rustup_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "rustup_path not set"))?
                .to_string()),
            "homebrew_path" => Ok(self.paths.homebrew_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "homebrew_path not set"))?
                .to_string()),
            "cmdline_tools_path" => Ok(self.paths.cmdline_tools_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cmdline_tools_path not set"))?
                .to_string()),
            "build_tools_path" => Ok(self.paths.build_tools_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "build_tools_path not set"))?
                .to_string()),
            "sdkmanager_path" => Ok(self.paths.sdkmanager_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "sdkmanager_path not set"))?
                .to_string()),
            "platform_tools_path" => Ok(self.paths.platform_tools_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "platform_tools_path not set"))?
                .to_string()),
            "platforms_path" => Ok(self.paths.platforms_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "platforms_path not set"))?
                .to_string()),
            "ndk_bundle_path" => Ok(self.paths.ndk_bundle_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "ndk_bundle_path not set"))?
                .to_string()),
            "java_path" => Ok(self.paths.java_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "java_path not set"))?
                .to_string()),
            "keystore_path" => Ok(self.paths.java_path
                .as_ref()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "keystore_path not set"))?
                .to_string()),
            _ => Err(io::Error::new(io::ErrorKind::NotFound, format!("Unknown Key: {}", key)))
        }

    }

}