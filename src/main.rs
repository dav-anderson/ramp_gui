use image::{self, imageops, DynamicImage, GenericImageView, ImageEncoder, ImageFormat};
use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Write, BufReader, BufRead, ErrorKind};
use std::path::Path;
use std::process::{Command, Stdio};
use std::fs::OpenOptions;
use regex::Regex;
use std::thread::sleep;
use std::time::{Duration, Instant};

struct Paths {
    sdk_path: Option<String>,
    ndk_path: Option<String>,
    cargo_path: Option<String>,
    cargo_apk_path: Option<String>,
    rustup_path: Option<String>,
    homebrew_path: Option<String>,
    cmdline_tools_path: Option<String>,
    sdkmanager_path: Option<String>,
    platform_tools_path: Option<String>,
    platforms_path: Option<String>,
    ndk_bundle_path: Option<String>,
    java_path: Option<String>,
    keystore_path: Option<String>,
}

struct Certs {
    macos: String
}

struct Session {
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
        let os = env::consts::OS.to_string();
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
            rustup_path: None,
            homebrew_path: None,
            cmdline_tools_path: None,
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
            "rustup_path" => self.paths.rustup_path = Some(file_path.clone()),
            "homebrew_path" => self.paths.homebrew_path = Some(file_path.clone()),
            "cmdline_tools_path" => self.paths.cmdline_tools_path = Some(file_path.clone()),
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
                    "rustup_path" => self.paths.rustup_path = Some(value.trim().to_string()),
                    "homebrew_path" => self.paths.homebrew_path = Some(value.trim().to_string()),
                    "cmdline_tools_path" => self.paths.cmdline_tools_path = Some(value.trim().to_string()),
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

//function to create the .ramp config file
fn create_ramp_config(session: &Session) -> io::Result<()> {
    let config_path = format!("{}/.ramp", session.home);

    //create the file if it doesn't exist
    if !Path::new(&config_path).exists(){
        File::create(&config_path)?;
    }

    Ok(())
}

// Function to check if a command is available in the system
fn is_command_available(cmd: &str) -> bool {
    Command::new("which") // Unix-like systems
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or_else(|_| {
            // On Windows, "which" isn't available, so try running the command directly
            Command::new(cmd)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
        })
}

// Function to install rustup
fn install_rustup(session: &mut Session) -> io::Result<()> {
    println!("Detected OS: {}", session.os);
    match session.os.as_str() {
        "linux" => {
            let mut attempts = 0;
            let max_attempts = 3;
            let mut success = false;
            while attempts < max_attempts && success == false {
                let mut apt_success = false;
                let mut curl_success = false;
                let mut unzip_success = false;
                attempts += 1;
                //update apt
                println!(
                    "running sudo apt update... attempt:{}",
                    attempts.to_string()
                );
                let apt_output = Command::new("sudo").args(["apt", "update"]).output()?;
                println!(
                    "apt update stdout: {}",
                    String::from_utf8_lossy(&apt_output.stdout)
                );
                if !apt_output.status.success() {
                    println!(
                        "apt update stderr: {}",
                        String::from_utf8_lossy(&apt_output.stderr)
                    );
                } else {
                    println!("apt success");
                    apt_success = true;
                }
                //install curl
                println!("installing curl...");
                let curl_output = Command::new("sudo")
                    .args(["apt", "install", "curl", "-y"])
                    .output()?;
                if !curl_output.status.success() {
                    println!(
                        "failed to install curl, stderr: {}",
                        String::from_utf8_lossy(&curl_output.stderr)
                    );
                } else {
                    println!("curl success");
                    curl_success = true;
                }
                //install unzip
                println!("installing unzip...");
                let unzip_output = Command::new("sudo")
                    .args(["apt", "install", "unzip", "-y"])
                    .output()?;
                if !unzip_output.status.success() {
                    println!(
                        "failed to install unzip, stderr: {}",
                        String::from_utf8_lossy(&unzip_output.stderr)
                    );
                } else {
                    println!("unzip success");
                    unzip_success = true;
                }
                if unzip_success == true && curl_success == true && apt_success == true {
                    success = true;
                    println!("********apt loop success******")
                }
            }
            if success == false {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to apt update & install curl and unzip",
                ));
            }

            println!("Downloading and installing rustup...");
            let status = Command::new("sh")
                .arg("-c")
                .arg("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y")
                .status()?;

            if !status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to install rustup",
                ));
            }
        }
        "macos" => {
            println!("Downloading and installing rustup...");
            let status = Command::new("sh")
                .arg("-c")
                .arg("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y")
                .status()?;

            if !status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to install rustup",
                ));
            }
            let sudo_user = env::var("SUDO_USER").unwrap().to_string();
            let permissions = Command::new("sudo").args(["chown", "-R", &sudo_user, &format!("{}/.cargo", session.home)]).output()?;
            if !permissions.status.success(){
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to enable permissions for .cargo directory"));
            }
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unsupported operating system",
            ));
        }
    }

    println!("rustup installed successfully!");
    Ok(())
}

// Function to ensure full Rust toolchain is installed via rustup
fn install_rust_toolchain(session: &mut Session) -> io::Result<()> {
    let status = Command::new(&session.get_path("rustup_path")?)
        .args(&["toolchain", "install", "stable"])
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to install Rust stable toolchain",
        ));
    }
    
    let sudo_user = env::var("SUDO_USER").unwrap().to_string();
    let permissions = Command::new("sudo").args(["chown", "-R", &sudo_user, &format!("{}/.cargo", session.home)]).output()?;
    if !permissions.status.success(){
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to enable permissions for .cargo directory"));
    }

    println!("Rust stable toolchain installed!");
    Ok(())
}

//install build targets for all supported ramp outputs
fn install_build_targets(session: &mut Session) -> io::Result<()> {
    println!("Detected OS: {}", session.os);

    let mac_targets: Vec<String> = vec![
        "aarch64-apple-ios".to_string(),
        "x86_64-apple-ios".to_string(),
        "aarch64-apple-ios-sim".to_string(),
        "x86_64-apple-darwin".to_string(),
        "aarch64-apple-darwin".to_string(),
    ];
    let targets: Vec<String> = vec![
        "x86_64-unknown-linux-gnu".to_string(),
        "aarch64-unknown-linux-gnu".to_string(),
        "aarch64-linux-android".to_string(),
        "i686-linux-android".to_string(),
        "x86_64-linux-android".to_string(),
        "x86_64-pc-windows-gnu".to_string(),
        "wasm32-unknown-unknown".to_string(),
    ];

    //get list of current installations
    let output = Command::new(&session.get_path("rustup_path")?)
        .args(&["target", "list", "--installed"])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to list installed rustup targets",
        ));
    }

    let installed = String::from_utf8_lossy(&output.stdout);
    println!("Currently installed targets: \n{}", installed);

    for target in targets {
        if !installed.contains(&target) {
            println!("Build target {} not found. Installing...", target);
            let status = Command::new(&session.get_path("rustup_path")?)
                .args(&["target", "add", &target])
                .status()?;

            if !status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to install target: {}", target),
                ));
            }
            println!("Installed {} successfully", target);
        } else {
            println!("Target: {} already installed", target);
        }
    }
    //macos only
    if session.os.as_str() == "macos" {
        for target in mac_targets {
            if !installed.contains(&target) {
                println!("Build target {} not found. Installing...", target);
                let status = Command::new(&session.get_path("rustup_path")?)
                    .args(&["target", "add", &target])
                    .status()?;

                if !status.success() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("failed to install target: {}", target),
                    ));
                }
                println!("Installed {} successfully", target);
            } else {
                println!("Target: {} already installed", target);
            }
        }
    }
    println!("Build targets installed!");
    Ok(())
}

fn install_homebrew(session: &mut Session) -> io::Result<()> {
    if &session.os.as_str() != &"macos"{
        println!("skipping homebrew, not mac");
        return Ok(())
    }
    println!("homebrew installation not found, installing homebrew");
    let brew_dir = if cfg!(target_arch = "aarch64"){
        "/opt/homebrew"
    }else{
        "/usr/local"
    };
    let brew_bin = format!("{}/bin", brew_dir);

    // Check if Homebrew is installed (check for brew in ~/homebrew/bin)
    let brew_ok = Path::new(&format!("{}/brew", brew_bin)).exists()
        && Command::new(&format!("{}/brew", brew_bin))
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
    if brew_ok {
        println!("Homebrew is already installed at {}. Skipping installation.", brew_dir);
    } else {
        // Check for curl
        if !Command::new("curl")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "curl not found. Please install curl manually.",
            ));
        }

        // Check for unzip (assuming tar is available, as it's built into macOS)
        if !Command::new("tar")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "tar not found. Please install tar manually.",
            ));
        }

        // Install Homebrew
        println!("Installing Homebrew to {}...", brew_dir);
        // Create homebrew directory
        fs::create_dir_all(&brew_dir)?;
        // Download and extract Homebrew tarball
        let tarball_url = "https://github.com/Homebrew/brew/tarball/master";
        let install_output = Command::new("bash")
            .arg("-c")
            .arg(format!(
                "curl -L {} | tar xz --strip 1 -C {}",
                tarball_url, brew_dir
            ))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
        if !install_output.status.success() {
            println!("Homebrew install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to install Homebrew"));
        }
        println!("Homebrew installed successfully to {}.", brew_dir);
    }

    let sudo_user = env::var("SUDO_USER").unwrap().to_string();
    //set permissions for homebrew
    let permissions = Command::new("sudo").args(["chown", "-R", &sudo_user, &brew_dir]).output()?;
    if !permissions.status.success(){
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to enable permissions for Homebrew installation directory"));
    }
    // Set PATH for homebrew in config file
    session.set_path("homebrew_path", format!("{}/brew", brew_bin))?;

    Ok(())
}

fn install_xcode_prompt() -> io::Result<()> {
    // Open App Store to Xcode page
    let output = Command::new("open")
        .args(["-a", "safari", "https://apps.apple.com/us/app/xcode/id497799835"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to open App Store: {}", e)))?;

    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to open Xcode page in App Store"));
    }

    //loop until Xcode is installed
    println!("Checking for Xcode installation every 5 seconds...");
    loop {
        if Path::new("/Applications/Xcode.app").exists() {
            // Verify Xcode installation by checking version
            let output = Command::new("/Applications/Xcode.app/Contents/MacOS/Xcode")
                .arg("--version")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to check Xcode version: {}", e)))?;

            if output.status.success() {
                println!("Xcode installed successfully!");
                return Ok(());
            }
        }

        // wait 5 seconds
        sleep(Duration::from_secs(5));
    }
}

fn install_macos_ios_toolchains(session: &mut Session) -> io::Result<()> {
    if session.os.as_str() != "macos" {
        println!("not on macos, skipping ios/macos toolchain installation");
        return Ok(())
    }
    //verify that the Xcode app is already installed
    println!("checking for xcode installation...");
    let xcode_app = "/Applications/Xcode.app";
    if !Path::new(xcode_app).exists() {
        install_xcode_prompt()?;
    }else{
        println!("xcode is already installed!");
    }
    //point xcode-select to the proper path
    Command::new("sudo").args([
        "xcode-select", "-s", "/Applications/Xcode.app/Contents/Developer"
    ])
    .status()?;

    if session.os.as_str() != "macos"{
        println!("skipping macos & ios toolchain install");
    }else{
        println!("installing ios and macos toolchains");
    }

    let brew_dir = if cfg!(target_arch = "aarch64"){
        "/opt/homebrew"
    }else{
        "/usr/local"
    };

    let brew_bin = format!("{}/bin", brew_dir);

    // Check if Homebrew is installed
    let brew_ok = Command::new(format!("{}/brew", &brew_bin))
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !brew_ok {
        //install homebrew if mac
        install_homebrew(session)?;
    } else {
        //update config to the bin path if already installed
        session.set_path("homebrew_path", brew_bin)?;
    }

    // Check for Xcode Command Line Tools
    let xcode_ok = Command::new("xcode-select")
        .arg("-p")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !xcode_ok {
        println!("Installing Xcode Command Line Tools...");
        // Install Xcode Command Line Tools
        if is_xcode_tools_installed() {
            println!("Command Line Tools for Xcode are already installed.");
            return Ok(());
        }
    
        println!("Command Line Tools for Xcode not found. Installing from softwareupdate...");
    
        // Create temporary file to signal softwareupdate
        let temp_file = "/tmp/.com.apple.dt.CommandLineTools.installondemand.in-progress";
        File::create(temp_file)?;
    
        // List available updates and find Command Line Tools
        let output = Command::new("softwareupdate")
            .arg("-l")
            .output()?;
    
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "softwareupdate -l failed"));
        }
    
        let output_str = String::from_utf8_lossy(&output.stdout);
        let prod_line = output_str
            .lines()
            .filter(|line| line.contains("*") && line.contains("Command Line"))
            .last()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No Command Line Tools package found"))?;
    
        // Extract package name (e.g., "Command Line Tools for Xcode-15.4")
        let prod = prod_line
            .split(" ")
            .skip_while(|s| *s != "Command")
            .collect::<Vec<&str>>()
            .join(" ");
    
        if prod.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Failed to parse Command Line Tools package name"));
        }
    
        // Install the package
        println!("Installing {}", prod);
        let install_status = Command::new("softwareupdate")
            .args(&["-i", &prod, "--verbose"])
            .status()?;
    
        // Clean up temporary file
        if Path::new(temp_file).exists() {
            std::fs::remove_file(temp_file)?;
        }
    
        if !install_status.success() {
            println!("Installation failed for {}", prod);
            return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to install {}", prod)));
        }
    
        // Verify installation
        if is_xcode_tools_installed() {
            println!("Command Line Tools for Xcode installed successfully!");
        } else {
            println!("Installation completed but verification failed");
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to verify Xcode installation"));
        }

        //Download Xcode IOS SDK
        let ios_sdk = Command::new("xcodebuild").args(["-downloadPlatform", "iOS"]).output()?;
        if !ios_sdk.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to install Xcode IOS SDK"));
        }
    }
    // Accept Xcode license (requires sudo)
    println!("Accepting Xcode license...");
    let status = Command::new("sudo")
        .args(&["xcodebuild", "-license", "accept"])
        .status()
        .expect("Failed to accept Xcode license");
    println!("Xcodebuild license accept resulsts: {:?}", status);
    if !status.success() {
        eprintln!("Failed to accept Xcode license.");
        std::process::exit(1);
    }

    //install libimobiledevice
    let output = Command::new(format!("{}/brew", session.paths.homebrew_path.as_ref().unwrap()))
        .args(["install", "libimobiledevice"])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to install libimobiledevice"));
    }

    println!("MacOS & IOS toolchain installation complete");
    Ok(())
}

fn install_simulators(session: &Session) -> io::Result<()>{
    if session.os.as_str() == "macos"{
        //run xcrun simctl list devices to initialize
        println!("setting up simulators");
        println!("setting up xcrun simctl");
        let output = Command::new("sudo")
        .args(["xcrun", "simctl", "list", "devices"])
        .output()
        .unwrap();
        println!("result: {:?}", output);
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to initialize xcode simulator"));
        }
    }
    
    Ok(())
}

fn setup_keychain(session: &mut Session) -> io::Result<()>{
    //TODO this currently creates a debug signing certificate only, different certificate properties must be configured for release on apple's developer website
    println!("keychain installer");
    if session.os.as_str() == "macos"{
        //check if keychain is locked, if so, unlock
        loop{
            let output = Command::new("security")
                .args(["show-keychain-info", &format!("{}/login.keychain-db", session.paths.keystore_path.as_ref().unwrap())])
                .output()
                .unwrap();
            
            if output.status.success() && !String::from_utf8_lossy(&output.stdout).contains("locked") {
                break;
            }else{
                // wait 3 seconds
                sleep(Duration::from_secs(3));
            }
        }

        //TODO check if identity exists, if so check if certificate is trusted, if so return Ok(())?
        //security find-identity?

        //TODO take in the users email and pass it into this function
        //FOR now use this terminal input placeholder
        let mut email = String::new();
        println!("*********************************************");
        println!("Please enter your apple developer email. This email must be associated with an apple developer account.");
        println!("*********************************************");
        io::stdin()
            .read_line(&mut email)
            .expect("failed to read line");
        let email = email.trim();
        println!("using email: {}", &email);

        let mut full_name = String::new();
        println!("*********************************************");
        println!("Please enter your full name. This must be the legal name associated with your apple developer account.");
        println!("*********************************************");
        io::stdin()
            .read_line(&mut full_name)
            .expect("failed to read line");
        let full_name = full_name.trim();
        println!("using name: {}", &full_name);

        let mut org = String::new();
        println!("*********************************************");
        println!("Please enter your organization's name. This must be the organization's name associated with your apple developer account. (You may leave this blank if indvidual)");
        println!("*********************************************");
        io::stdin()
            .read_line(&mut org)
            .expect("failed to read line");
        let org = org.trim();
        println!("using name: {}", &org);

        println!("setting up keychain for macos");
        //check if the key already exists and if it does do not generate a new one
        match &session.paths.keystore_path {
            Some(val) => {
                println!("keystore path is already set");
            },
            None => {
                println!("keystore path not set");
                session.set_path("keystore_path", format!("{}/Library/Keychains", session.home))?;
            },
        }
        //check if the keypath exists, if not generate a new key
        let key_path = format!("{}/ramp.pem", session.paths.keystore_path.as_ref().unwrap());
        if !Path::new(&key_path).exists() {
            println!("no private key found, generating new key");
             //generate the signing key
            let output = Command::new("openssl")
            .args([
                "genrsa",
                "-out",
                &key_path,
                "2048"
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to generate private key: {}", e)))?;
            if !output.status.success() {
                return Err(io::Error::new(io::ErrorKind::Other, "OpenSSL key generation failed"));
            }
            //security import the private key into the keychain
            let output = Command::new("security")
            .args(["import", &format!("{}/ramp.pem", session.paths.keystore_path.as_ref().unwrap()), "-k", &format!("{}/login.keychain-db", session.paths.keystore_path.as_ref().unwrap()), "-T", "/usr/bin/codesign"])
            .output()
            .unwrap();
            if !output.status.success() {
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to import the ramp.pem to the keychain-db"));
            }
        }else {println!("ramp.pem already exists");}
        //check if the CSR exists, if not generate a new CSR
        let csr_path = format!("{}/ramp.csr", session.paths.keystore_path.as_ref().unwrap());
        if !Path::new(&csr_path).exists() {
            let mut subject = String::new();
            if org == "" {
                subject = format!("/CN={} /emailAddress={}", full_name, email);
            } else {
                subject = format!("/CN={} /O={} /emailAddress={}", full_name, org, email);
            }
            // Generate a CSR
            let output = Command::new("openssl")
            .args([
                "req",
                "-new",
                "-key",
                &key_path,
                "-out",
                &csr_path,
                "-subj",
                &subject,
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to generate CSR: {}", e)))?;
            if !output.status.success() {
                return Err(io::Error::new(io::ErrorKind::Other, "OpenSSL CSR generation failed"));
            }
        }else{println!("ramp.csr already exists");}

        //open apple developer portal
        let output = Command::new("open")
            .args(["-a", "safari", "https://developer.apple.com/account/resources/certificates/list"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to open Apple developer portal: {}", e)))?;
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to open Apple developer portal"));
        }       
        //open the file explorer to show the CSR
        let output = Command::new("open")
            .arg(session.paths.keystore_path.as_ref().unwrap())
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to open CSR in file explorer at path {}", &csr_path)));
        }
        //TODO eventually replace these prints in the GUI
        println!("1. Now go to the safari window and login to your developer account.");
        println!("2. Once you have logged in, click the + button next to \"Certificates\".");
        println!("3. Select the \"Apple Development\" checkbox and then hit next.");
        println!("4. Next drag and drop the \"ramp.csr\" file from the file finder window into Safari. Click the next button.");
        println!("5. Click the download button");

        //continuously checks for "development.cer" in the Downloads folder and moves it once found
        let cert_download = format!("{}/Downloads/development.cer", session.home);
        loop {
            if Path::new(&cert_download).exists() {
                //Copy the cert to the keychain
                let output = Command::new("mv")
                    .args([&cert_download, session.paths.keystore_path.as_ref().unwrap()])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .output()
                    .unwrap();
                if !output.status.success() {
                    return Err(io::Error::new(io::ErrorKind::Other, "Failed to move the development.cer to the keychain directory"));
                }
                println!("Successfully downloaded signing certificate!");
                break;
            }else {
                // wait 3 seconds
                sleep(Duration::from_secs(3));
            }
        }
        //TODO can probably remove this was using for testing
        //convert the .cer to a .pem
        // let output = Command::new("openssl")
        //     .args(["x509", "-in", &format!("{}/development.cer", session.paths.keystore_path.as_ref().unwrap()), "-inform", "DER", "-out", &format!("{}/development.pem", session.paths.keystore_path.as_ref().unwrap()), "-outform", "PEM"])
        //     .output()
        //     .unwrap();
        // if !output.status.success() {
        //     return Err(io::Error::new(io::ErrorKind::Other, "Failed to convert the development.cer to development.pem"));
        // }
        //security import the cert into the keychain
        let output = Command::new("security")
            .args(["import", &format!("{}/development.cer", session.paths.keystore_path.as_ref().unwrap()), "-k", &format!("{}/login.keychain-db", session.paths.keystore_path.as_ref().unwrap())])
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to import the development.pem to the keychain-db"));
        }
        //TODO this may be deprecated and/or causing a signing bug
        //TODO test if this works with a fresh install without a trusted cert
        // add the certificate as trusted for code signing
        // let output = Command::new("security")
        //     .args(["add-trusted-cert", "-d", "-r", "trustAsRoot", "-k", &format!("{}/login.keychain-db", session.paths.keystore_path.as_ref().unwrap()), &format!("{}/development.cer", session.paths.keystore_path.as_ref().unwrap())])
        //     .output()
        //     .unwrap();
        // if !output.status.success() {
        //     return Err(io::Error::new(io::ErrorKind::Other, "Failed to add trusted cert to the development.cer"));
        // }        
        //get the App Developer Worldwide Developer Relations Ceritifcation Authority certificate
        let output = Command::new("curl")
            .args(["-o", &format!("{}/AppleWWDRCA.cer", session.paths.keystore_path.as_ref().unwrap()), "https://www.apple.com/certificateauthority/AppleWWDRCAG3.cer"])
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to obtain Apple WWDRCA cert with curl"));
        }  
        //add the apple Developer worldwide relations cert to the security chain
        let output = Command::new("security")
        .args(["import", &format!("{}/AppleWWDRCA.cer", session.paths.keystore_path.as_ref().unwrap()), "-k", &format!("{}/login.keychain-db", session.paths.keystore_path.as_ref().unwrap())])
        .output()
        .unwrap();
        println!("AppleWWDRCA.cer import output: {:?}", output);
        if !output.status.success() && !String::from_utf8_lossy(&output.stderr).contains("already exists") {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to import the AppleWWDRCA.cer to the keychain-db"));
        }
        //get the App Developer Worldwide Developer Relations Ceritifcation Authority certificate
        let output = Command::new("curl")
        .args(["-o", &format!("{}/AppleRootCA.cer", session.paths.keystore_path.as_ref().unwrap()), "https://www.apple.com/certificateauthority/AppleRootCA-G3.cer"])
        .output()
        .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to obtain AppleRootCA cert with curl"));
        }  
        //add the apple Root CA cert to the security chain
        let output = Command::new("security")
        .args(["import", &format!("{}/AppleRootCA.cer", session.paths.keystore_path.as_ref().unwrap()), "-k", &format!("{}/login.keychain-db", session.paths.keystore_path.as_ref().unwrap())])
        .output()
        .unwrap();
        println!("AppleRootCA.cer import output: {:?}", output);
        if !output.status.success() && !String::from_utf8_lossy(&output.stderr).contains("already exists") {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to import the AppleRootCA.cer to the keychain-db"));
        }
        //Get the Developer ID CA
        let output = Command::new("curl")
        .args(["-o", &format!("{}/AppleDevIDCA.cer", session.paths.keystore_path.as_ref().unwrap()), "https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer"])
        .output()
        .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to obtain AppleDevIDCA.cer cert with curl"));
        }  
        //add the apple Root CA cert to the security chain
        let output = Command::new("security")
        .args(["import", &format!("{}/AppleDevIDCA.cer", session.paths.keystore_path.as_ref().unwrap()), "-k", &format!("{}/login.keychain-db", session.paths.keystore_path.as_ref().unwrap())])
        .output()
        .unwrap();
        println!("AppleDevIDCA.cer import output: {:?}", output);
        if !output.status.success() && !String::from_utf8_lossy(&output.stderr).contains("already exists") {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to import the AppleDevIDCA.cer to the keychain-db"));
        }

        println!("Successfully set up the keychain for macos!")

    }
    //TODO add support for all other target builds
    Ok(())
}

//sign an app build
fn sign_build(session: &mut Session, target_os: String, release: bool) -> io::Result<()> {
    println!("signing app bundle for {}", target_os);
    if target_os == "ios" {
        //check if keychain is locked, if so, unlock
        loop{
            let output = Command::new("security")
                .args(["show-keychain-info", &format!("{}/login.keychain-db", session.paths.keystore_path.as_ref().unwrap())])
                .output()
                .unwrap();
            
            if output.status.success() && !String::from_utf8_lossy(&output.stdout).contains("locked") {
                break;
            }else{
                // wait 3 seconds
                sleep(Duration::from_secs(3));
            }
        }
        //sign the build
        let output = Command::new("codesign")
        .args(["--force", "--sign", session.certs.macos.as_ref(), &format!("{}/{}/ios/{}.app", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), capitalize_first(session.current_project.as_ref().unwrap()))])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("ios post build failed to sign app bundle: {}", error),
            ));
        }
    }
    //TODO add support for other outputs
    println!("signed {} app bundle", target_os);
    Ok(())
}

fn is_xcode_tools_installed() -> bool {
    Command::new("xcode-select")
        .arg("-p")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn get_user_home() -> io::Result<String> {
    // Try SUDO_USER first to get the invoking user's home directory
    if let Ok(sudo_user) = env::var("SUDO_USER") {
        Ok(format!("/Users/{}", sudo_user))
    } else {
        // Fall back to $HOME if not running with sudo
        env::var("HOME")
            .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Failed to get HOME: {}", e)))
    }
}

fn install_android_toolchains(session: &mut Session) -> io::Result<()> {
    println!("Setting up Android SDK and NDK for {}", session.os);
    session.set_path("sdk_path", format!("{}/Android/sdk", session.home))?;
    session.set_path("cmdline_tools_path", format!("{}/Android/sdk/cmdline-tools", session.home))?;
    session.set_path("sdkmanager_path", format!("{}/Android/sdk/cmdline-tools/bin/sdkmanager", session.home))?;
    session.set_path("platform_tools_path", format!("{}/Android/sdk/platform-tools", session.home))?;
    session.set_path("platforms_path", format!("{}/platforms/android-{}", format!("{}/Android/sdk", session.home), session.android_platform_version))?;
    session.set_path("ndk_path", format!("{}/Android/sdk/ndk/{}", session.home, session.android_ndk_version))?;
    session.set_path("ndk_bundle_path", format!("{}/Android/sdk/ndk-bundle", session.home))?;

    // Check for JDK
    session.set_path("java_path", "/opt/homebrew/opt/openjdk@17/bin/java".to_string())?;
    println!("Java path: {}", session.get_path("java_path")?);
    let java_ok = match Command::new(&session.get_path("java_path")?)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output() {
            Ok(output) => String::from_utf8_lossy(&output.stderr)
                .to_lowercase()
                .contains("openjdk"),
            Err(_) => false,
        };
 

    // OS-specific configuration
    let (java_home, sdk_url, install_jdk): (
        &str,
        &str,
        Box<dyn Fn() -> io::Result<()>>,
    ) = match session.os.as_str() {
        "linux" => {
            let java_home = session.get_path("java_path")?;
            println!("Java home: {}", java_home.to_string());
            (
                "/usr/lib/jvm/java-17-openjdk-amd64",
                "https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip",
                Box::new(|| -> io::Result<()> {
                    println!("Installing OpenJDK 17...");
                    let update_output = Command::new("sudo")
                        .args(&["bash", "-c", "apt update"])
                        .output()?;
                    if !update_output.status.success() {
                        println!("apt update stderr: {}", String::from_utf8_lossy(&update_output.stderr));
                        return Err(io::Error::new(io::ErrorKind::Other, "Failed to run apt update"));
                    }
                    let install_output = Command::new("sudo")
                        .args(&["bash", "-c", "apt install -y openjdk-17-jdk"])
                        .output()?;
                    if !install_output.status.success() {
                        println!("apt install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
                        return Err(io::Error::new(io::ErrorKind::Other, "Failed to install OpenJDK 17"));
                    }
                    Ok(())
                }),
            )
        },
        "macos" => {
            let sudo_user = env::var("SUDO_USER").map_err(|_| io::Error::new(io::ErrorKind::NotFound, "SUDO USER NOT FOUND"))?;
            let java_home = if cfg!(target_arch = "aarch64") {
                "/opt/homebrew/opt/openjdk@17"
            } else {
                "/usr/local/opt/openjdk@17"
            };
            session.set_path("java_path", java_home.to_string())?;
            (
                java_home,
                "https://dl.google.com/android/repository/commandlinetools-mac-11076708_latest.zip",
                Box::new(move || -> io::Result<()> {
                    println!("Installing OpenJDK 17...");
                    let brew_path = if java_home.starts_with("/opt/homebrew") {
                        "/opt/homebrew/bin/brew"
                    } else {
                        "/usr/local/bin/brew"
                    };
                    let install_output = Command::new("su")
                        .args(&[&sudo_user, "-c", &format!("{} install openjdk@17", brew_path)])
                        .output()?;
                    if !install_output.status.success() {
                        println!("brew install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
                        return Err(io::Error::new(io::ErrorKind::Other, "Failed to install OpenJDK 17"));
                    }
                    Ok(())
                }),
            )
        },
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unsupported OS: {}", session.os),
            ));
        }
    };

    // Install JDK if needed
    if !java_ok || java_home.is_empty() {
        println!(
            "JDK not found or JAVA_HOME not set. java_home: {}, java_ok: {}",
            java_home, java_ok
        );
        install_jdk()?;
        println!("Installed OpenJDK 17 and set JAVA_HOME={}", java_home);
    } else {
        println!("JDK found with JAVA_HOME={}", java_home);
    }

    // Check for existing SDK/NDK configuration
    let sdk_configured = {

        let sdkmanager_ok = Path::new(&session.get_path("sdkmanager_path")?).exists();
        let platform_tools_ok = Path::new(&session.get_path("platform_tools_path")?).exists();
        let platforms_ok = Path::new(&session.get_path("platforms_path")?).exists();
        let ndk_ok = Path::new(&session.get_path("ndk_path")?).exists() || Path::new(&session.get_path("ndk_bundle_path")?).exists();
        if !ndk_ok {
            let ndk_dir = format!("{}/ndk", session.get_path("sdk_path")?);
            if Path::new(&ndk_dir).exists() {
                let versions = fs::read_dir(&ndk_dir)?
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.path().is_dir())
                    .map(|entry| entry.file_name().into_string().unwrap_or_default())
                    .collect::<Vec<_>>();
                println!("Available NDK versions: {:?}", versions);
            }
        }
        println!(
            "SDK checks: sdkmanager={} ({}), platform_tools={} ({}), platforms={} ({}), ndk={} ({})",
            sdkmanager_ok, session.get_path("sdkmanager_path")?,
            platform_tools_ok, session.get_path("platform_tools_path")?,
            platforms_ok, session.get_path("platforms_path")?,
            ndk_ok, session.get_path("ndk_path")?
        );
        sdkmanager_ok && platform_tools_ok && platforms_ok && ndk_ok
    };

    if !sdk_configured {
        // Download and install command-line tools
        println!("Installing Android command-line tools...");
        let download_path = format!("{}/cmdline-tools.zip", session.home);
        Command::new("curl")
            .args(&["-o", &download_path, sdk_url])
            .status()?;
        Command::new("mkdir")
            .args(&["-p", &session.get_path("sdk_path")?])
            .status()?;
        Command::new("unzip")
            .args(&["-o", &download_path, "-d", &session.get_path("sdk_path")?])
            .status()?;
        Command::new("rm")
            .arg(&download_path)
            .status()?;
        // Accept licenses
        println!("Accepting Android SDK licenses...");
        let mut license_cmd = Command::new("yes")
            .stdout(Stdio::piped())
            .spawn()?;
        let license_output = Command::new(&session.get_path("sdkmanager_path")?)
            .args(["--licenses", &format!("--sdk_root={}", &session.get_path("sdk_path")?)])
            .env("JAVA_HOME", &session.get_path("java_path")?)
            .stdin(license_cmd.stdout.take().unwrap())
            .output()?;
        license_cmd.wait()?;
        if !license_output.status.success() {
            println!("License acceptance stderr: {}", String::from_utf8_lossy(&license_output.stderr));
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to accept Android SDK licenses"));
        }
        // Install SDK and NDK packages
        let ndk_package = format!("ndk;{}", session.android_ndk_version);
        let platform_package = format!("platforms;android-{}", session.android_platform_version);
        let packages = vec!["platform-tools", "build-tools;34.0.0", &platform_package, &ndk_package];
        for package in packages {
            println!("Installing {}...", package);
            let install_output = Command::new(&session.get_path("sdkmanager_path")?)
                .args(&[package, &format!("--sdk_root={}", &session.get_path("sdk_path")?)])
                .env("JAVA_HOME", &session.get_path("java_path")?)
                .output()?;
            if !install_output.status.success() {
                println!("Install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
                return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to install {}", package)));
            }
        }
        println!("Android SDK and NDK installed.");
    } else {
        println!("Existing Android SDK and NDK found at {}. Skipping installation.", session.get_path("sdk_path")?);
    }
    // Install cargo-apk
    session.set_path("cargo_apk_path", format!("{}/.cargo/bin/cargo-apk", session.home))?;
    let cargo_apk_ok = Command::new(&session.get_path("cargo_apk_path")?)
        .args(&["apk", "version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if cargo_apk_ok {
        println!("cargo-apk is already installed. Skipping installation.");
    } else {
        // Install cargo-apk
        println!("Installing cargo-apk...");
        let install_output = Command::new(session.get_path("cargo_path")?)
            .args(&["install", "cargo-apk"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
        if !install_output.status.success() {
            println!("cargo install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to install cargo-apk"));
        }
        println!("cargo-apk installed successfully.");
    }

    Ok(())
}

fn new_project(session: &mut Session, name: &str) -> io::Result<()> {
    //check network connectivity
    println!("Checking for network connectivity...");
    //ping linux servers once to check for connectivity
    let output = Command::new("ping")
        .args(["-c", "1", "linux.org"])
        .output()
        .unwrap();
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "No network connection detected",
        ));
    }
    let new_path = format!(
        "{}/{}",
        session.projects_path.as_ref().unwrap_or(&String::new()),
        name.to_lowercase()
    );
    println!("the new path is: {}", &new_path);
    //prepare the template at the target path
    match session.os.as_str() {
        "linux" => {
            // Ensure git is installed
            if !is_command_available("git") {
                let mut success = false;
                let mut attempt = 0;
                let max_attempts = 3;
                while success == false && attempt < max_attempts {
                    attempt += 1;
                    println!("git not found. Installing git...");
                    let git_output = Command::new("bash")
                        .args(&["-c", "apt install -y git"])
                        .output()?;
                    println!(
                        "git install stdout: {}",
                        String::from_utf8_lossy(&git_output.stdout)
                    );
                    if !git_output.status.success() {
                        println!(
                            "git install stderr: {}",
                            String::from_utf8_lossy(&git_output.stderr)
                        );
                    } else {
                        success = true;
                    }
                }
                if success == false {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Failed to install git",
                    ));
                }
            }

            // Create the parent directory if it doesn't exist
            if !Path::new(&new_path).exists() {
                println!("Creating directory: {}", &new_path);
                let mkdir_output = Command::new("mkdir").args(&["-p", &new_path]).output()?;
                if !mkdir_output.status.success() {
                    println!(
                        "mkdir stderr: {}",
                        String::from_utf8_lossy(&mkdir_output.stderr)
                    );
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Failed to create projs directory",
                    ));
                }
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Project by that name already exists",
                ));
            }

            // Clone the template repository
            println!(
                "Cloning template from https://github.com/dav-anderson/ramp_template to {}",
                &new_path
            );
            let clone_output = Command::new("git")
                .args(&[
                    "clone",
                    "https://github.com/dav-anderson/ramp_template",
                    &new_path,
                ])
                .output()?;
            println!(
                "git clone stdout: {}",
                String::from_utf8_lossy(&clone_output.stdout)
            );
            if !clone_output.status.success() {
                println!(
                    "git clone stderr: {}",
                    String::from_utf8_lossy(&clone_output.stderr)
                );
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to clone template repository",
                ));
            }

            println!("Template cloned successfully to {}", &new_path);
        }

        "macos" =>  {
            // Create the parent directory if it doesn't exist
            if !Path::new(&new_path).exists() {
                println!("Creating directory: {}", &new_path);
                let mkdir_output = Command::new("mkdir").args(&["-p", &new_path]).output()?;
                if !mkdir_output.status.success() {
                    println!(
                        "mkdir stderr: {}",
                        String::from_utf8_lossy(&mkdir_output.stderr)
                    );
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Failed to create projs directory",
                    ));
                }
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Project by that name already exists",
                ));
            }

            // Clone the template repository
            println!(
                "Cloning template from https://github.com/dav-anderson/ramp_template to {}",
                &new_path
            );
            let clone_output = Command::new("git")
                .args(&[
                    "clone",
                    "https://github.com/dav-anderson/ramp_template",
                    &new_path,
                ])
                .output()?;
            println!(
                "git clone stdout: {}",
                String::from_utf8_lossy(&clone_output.stdout)
            );
            if !clone_output.status.success() {
                println!(
                    "git clone stderr: {}",
                    String::from_utf8_lossy(&clone_output.stderr)
                );
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to clone template repository",
                ));
            }

            println!("Template cloned successfully to {}", &new_path);
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unsupported OS for cloning template",
            ))
        }
    }

    //create bundle identifier
    let bundle_id = create_app_bundle_id(session)?;

    //rename everything inside of the template with the project name
    template_naming(session, &name.to_lowercase(), bundle_id)?;

    //update the current loaded project to the new project
    load_project(session, &name.to_lowercase())?;

    Ok(())
}

fn load_project(session: &mut Session, name: &str) -> io::Result<()> {
    println!("loading project...");
    session.update_current_project(name.to_string())?;
    Ok(())
}

//renames all of the paths and file contents of the template to match the user provided name when creating a new ramp project
fn template_naming(session: &mut Session, name: &str, bundle_id: Option<String>) -> io::Result<()> {
    let new_path = format!(
        "{}/{}",
        session.projects_path.as_ref().unwrap_or(&String::new()),
        name
    );
    let capitalized_name = capitalize_first(name);
    let replacements = vec![("Webgpu", capitalized_name.as_str()), ("webgpu", name)];
    //rename default strings in cargo.toml
    replace_strings_in_file(&format!("{}/Cargo.toml", new_path), &replacements)?;
    //rename dir ios/Webgpu.app
    rename_directory(
        &format!("{}/ios/Webgpu.app", new_path),
        &format!("{}.app", &capitalized_name),
    )?;
    //rename default strings in ios/Webgpu.app/Info.plist
    replace_strings_in_file(
        &format!("{}/ios/{}.app/Info.plist", new_path, capitalized_name),
        &replacements,
    )?;
    //rename dir macos/Webgpu.app
    rename_directory(
        &format!("{}/macos/Webgpu.app", new_path),
        &format!("{}.app", &capitalized_name),
    )?;
    //rename default strings in macos/Webgpu.app/Contents/Info.plist
    replace_strings_in_file(
        &format!("{}/macos/{}.app/Info.plist", new_path, capitalized_name),
        &replacements,
    )?;
    //replace bundle id if applicable
    if bundle_id.is_some() {
        let existing_bundle = format!("com.ramp.{}", session.current_project.as_ref().unwrap());
        let replacements = vec![(existing_bundle.as_str(), bundle_id.as_ref().unwrap().as_str())];
        replace_strings_in_file(
            &format!("{}/ios/{}.app/Info.plist", new_path, capitalized_name),
            &replacements,
        )?;
    }
    //rename default strings in Cargo.toml
    let replacements = vec![("ramp_template", name)];
    replace_strings_in_file(
        &format!("{}/ios/{}.app/Info.plist", new_path, capitalized_name),
        &replacements,
    )?;    

    Ok(())
}

//renames a target directory to a given new String
fn rename_directory(current_path: &str, target_name: &str) -> io::Result<()> {
    // Get the parent directory of the current path
    let current_dir = Path::new(current_path);
    let parent_dir = current_dir.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Current path has no parent directory",
        )
    })?;

    // Construct the new path by joining the parent directory with the target name
    let new_path = parent_dir.join(target_name);

    // Rename the directory
    fs::rename(current_path, &new_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to rename {} to {}: {}",
                current_path,
                new_path.display(),
                e
            ),
        )
    })?;

    println!(
        "Renamed directory from {} to {}",
        current_path,
        new_path.display()
    );
    Ok(())
}

//find and replace target strings in a target file
fn replace_strings_in_file(file_path: &str, replacements: &Vec<(&str, &str)>) -> io::Result<()> {
    // Read the file content into a string
    let content = fs::read_to_string(file_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to read file {}: {}", file_path, e),
        )
    })?;

    // Perform all replacements
    let mut new_content = content;
    for &(find, replace) in replacements {
        new_content = new_content.replace(find, replace);
    }

    // Write the modified content back to the file
    fs::write(file_path, &new_content).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to write to file {}: {}", file_path, e),
        )
    })?;

    println!("Updated file {} with replacements", file_path);
    Ok(())
}

fn get_bundle_id(target_os: &str) -> io::Result<String> {
    let plist_path = format!("{}/Info.plist", target_os);
    let plist_content = fs::read_to_string(&plist_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to read Info.plist: {}", e)))?;

    let re = Regex::new(r#"<key>CFBundleIdentifier</key>\s*<string>([^<]+)</string>"#).unwrap();
    let captures = re.captures(&plist_content)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "CFBundleIdentifier not found"))?;

    let bundle_id = captures.get(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Failed to extract CFBundleIdentifier"))?
        .as_str()
        .to_string();

    Ok(bundle_id)
}

//capitalize the first letter in a string
fn capitalize_first(s: &str) -> String {
    match s.get(0..1) {
        None => String::new(),
        Some(first) => first.to_uppercase() + &s[1..],
    }
}

fn resize_png(input_name: &str, target_name: &str, width: u32, height: u32) -> io::Result<()> {
    // Open the input PNG file
    let img = image::open(input_name).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to open {}: {}", input_name, e),
        )
    })?;

    //remove target output if it exists
    if Path::new(&target_name).exists() {
        let output = Command::new("rm")
            .arg(&target_name)
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not remove old icon: {}",
            ));
        }
    }

    // Resize the image to the target resolution
    let resized_img = imageops::resize(&img, width, height, imageops::FilterType::Lanczos3);

    // Save the resized image to the target name
    resized_img.save(target_name).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to save {}: {}", target_name, e),
        )
    })?;

    println!(
        "Resized {} to {}x{} and saved as {}",
        input_name, width, height, target_name
    );
    Ok(())
}

fn convert_png_to_ico(session: &Session, input_path: &str) -> io::Result<()> {
    let windows = "windows_icon.ico";
    let favicon = "favicon.ico";
    let win_output_path = format!(
        "{}/{}/assets/resources/icons/{}",
        session.projects_path.as_ref().unwrap(),
        session.current_project.as_ref().unwrap(),
        windows
    );
    let wasm_output_path = format!(
        "{}/{}/assets/resources/icons/{}",
        session.projects_path.as_ref().unwrap(),
        session.current_project.as_ref().unwrap(),
        favicon
    );

    //remove old windows.ico if it exists
    if Path::new(&win_output_path).exists() {
        let output = Command::new("rm")
            .arg(&win_output_path)
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not remove old windows icon: {}",
            ));
        }
    }
    //remove old favicon if it exists
    if Path::new(&wasm_output_path).exists() {
        let output = Command::new("rm")
            .arg(&wasm_output_path)
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not remove old favicon: {}",
            ));
        }
    }
    // Open the PNG file
    let img = image::open(input_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to open {}: {}", input_path, e),
        )
    })?;

    // Resize to the specified size
    let resized = imageops::resize(&img, 64, 64, imageops::FilterType::Lanczos3);
    let resized_img = DynamicImage::ImageRgba8(resized);

    // Write as ICO
    let file = std::fs::File::create(win_output_path.clone())?;
    let mut writer = std::io::BufWriter::new(file);
    let encoder = image::codecs::ico::IcoEncoder::new(&mut writer);
    encoder
        .write_image(
            resized_img.as_bytes(),
            64,
            64,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to save {} as ICO: {}", win_output_path, e),
            )
        })?;
    println!(
        "Converted {} to ICO ({}x{}) and saved as {}",
        input_path, 64, 64, win_output_path
    );

    //check for app.rc and if it exists remove it
    let rc = format!(
        "{}/{}/app.rc",
        session.projects_path.as_ref().unwrap(),
        session.current_project.as_ref().unwrap()
    );
    if Path::new(&rc).exists() {
        let output = Command::new("rm").arg(&rc).output().unwrap();
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not remove old app.rc: {}",
            ));
        }
    }
    //create a new app.rc using absolute path passed in
    let ico_path = format!(
        "{}/{}/assets/resources/icons/windows_icon.ico",
        session.projects_path.as_ref().unwrap(),
        session.current_project.as_ref().unwrap()
    );
    let rc_content = format!(r#"IDI_ICON1 ICON "{}""#, ico_path);
    let mut rc_file = File::create(&rc)?;
    rc_file.write_all(rc_content.as_bytes())?;
    //ensure the file is fully written
    rc_file.flush()?;
    //explicitly close the file
    drop(rc_file);
    println!("created resource file: {}", &rc);
    let res = format!(
        "{}/{}/app.res",
        session.projects_path.as_ref().unwrap(),
        session.current_project.as_ref().unwrap()
    );
    println!("rc path: {}", &rc);
    println!("res path: {}", &res);
    let build_path = format!(
        "{}/{}/build.rs",
        session.projects_path.as_ref().unwrap(),
        session.current_project.as_ref().unwrap()
    );
    //if a build.rs file exists, first remove it.
    if Path::new(&build_path).exists() {
        let output = Command::new("rm")
            .arg(&build_path)
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not remove old build.rs",
            ));
        }
    }
    //populate the build.rs content
    let build_content = format!(
        r#"
        use std::io;

        fn main() {{
            if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" && std::path::Path::new("{}").exists() {{
                embed_resource::compile("app.rc", embed_resource::NONE)
                .manifest_optional();
            }}
    }}
    
        "#,
        &ico_path
    );
    //Generate a build.rs file
    let mut build_file = fs::File::create(&build_path)?;
    build_file.write_all(build_content.as_bytes())?;
    build_file.flush()?;
    println!("Created Build.rs at {}", &build_path);
    //copy windows_icon.ico into a favicon.ico
    let output = Command::new("cp")
        .args([&win_output_path, &wasm_output_path])
        .output()
        .unwrap();

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "could not copy favicon: {}",
        ));
    }
    println!(
        "copied {} ({}x{}) as {}",
        win_output_path, 64, 64, wasm_output_path
    );
    Ok(())
}

//update all of the icons in the project from a single image provided in <projects_path>/<project_name>/assets/resources/icons
//reccomended input is a 1024X1024 .png
fn update_icons(session: &Session) -> io::Result<()> {
    let originating_icon = format!(
        "{}/{}/assets/resources/icons/icon.png",
        session.projects_path.as_ref().unwrap(),
        session.current_project.as_ref().unwrap()
    );
    //update android icons
    resize_png(
        &originating_icon,
        &format!(
            "{}/{}/android/app/src/main/res/mipmap-mdpi/ic_launcher.png",
            session.projects_path.as_ref().unwrap(),
            session.current_project.as_ref().unwrap()
        ),
        48,
        48,
    )?;
    resize_png(
        &originating_icon,
        &format!(
            "{}/{}/android/app/src/main/res/mipmap-hdpi/ic_launcher.png",
            session.projects_path.as_ref().unwrap(),
            session.current_project.as_ref().unwrap()
        ),
        72,
        72,
    )?;
    resize_png(
        &originating_icon,
        &format!(
            "{}/{}/android/app/src/main/res/mipmap-xhdpi/ic_launcher.png",
            session.projects_path.as_ref().unwrap(),
            session.current_project.as_ref().unwrap()
        ),
        96,
        96,
    )?;
    resize_png(
        &originating_icon,
        &format!(
            "{}/{}/android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png",
            session.projects_path.as_ref().unwrap(),
            session.current_project.as_ref().unwrap()
        ),
        144,
        144,
    )?;
    resize_png(
        &originating_icon,
        &format!(
            "{}/{}/android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png",
            session.projects_path.as_ref().unwrap(),
            session.current_project.as_ref().unwrap()
        ),
        192,
        192,
    )?;

    //update windows icon
    convert_png_to_ico(&session, &originating_icon)?;

    //TODO add linux support
    if session.os.as_str() == "macos" {
        println!("update icons for macos/ios");
        resize_png(
            &originating_icon,
            &format!("{}/{}/ios/{}.app/Assets/ios_icon120.png", session.projects_path.as_ref().unwrap(),
            session.current_project.as_ref().unwrap(),
            capitalize_first(session.current_project.as_ref().unwrap())),
            120,
            120,
        )?;
        resize_png(
            &originating_icon,
            &format!("{}/{}/ios/{}.app/Assets/ios_icon180.png", session.projects_path.as_ref().unwrap(),
            session.current_project.as_ref().unwrap(),
            capitalize_first(session.current_project.as_ref().unwrap())),
            180,
            180,
        )?;
        println!("updating icons for macos");
        let macos = Command::new("sips").args(["-s", "format", "icns", &originating_icon, "--out", &format!("{}/{}/assets/resources/icons/macos_icon.icns", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap())]).output().unwrap();
        if !macos.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not create macos icns: {}",
            ));
        }
    }
    Ok(())
}

fn provision_device(session: &mut Session, uuid: String, target_os: &String, release: bool) -> io::Result<()> {
    println!("Provisioning a new device with id: {}", &uuid);
    //open apple developer portal
    let output = Command::new("open")
    .args(["-a", "safari", "https://developer.apple.com/account/resources/devices/list"])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .output()
    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to open Apple developer portal to devices list: {}", e)))?;
    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to open Apple developer portal to devices list"));
    }    
    println!("**********************************");   
    //TODO eventually replace these prints in the GUI
    println!("1. Now go to the safari window and login to your developer account.");
    println!("2. Once you have logged in, click the + button next to \"Devices\".");
    println!("3. Give the device whatever name you would like and copy and paste in the following numbers and letters into \"Device ID (UUID)\".");
    println!("{}", &uuid);
    println!("4. Click the continue button.");
    println!("5. Press enter here in the terminal to continue");
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to receive user input: {}", e)))?;

    //parse the app bundle id from the info.plist
    let path_string = format!("{}/{}/{}/{}.app/Info.plist", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), &target_os, capitalize_first(session.current_project.as_ref().unwrap()));
    let path = Path::new(&path_string);
    let content = fs::read_to_string(path).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("failed to read info.plist: {}", e)))?;
    let key_str = "<key>CFBundleIdentifier</key>";
    let key_pos = content.find(key_str).ok_or_else(|| io::Error::new(io::ErrorKind::Other, format!("CFBundleIdentifier key not found in Info.plist")))?;
    let start_after_key = key_pos + key_str.len();
    let rest_after_key = &content[start_after_key..];

    let string_open = "<string>";
    let string_pos = rest_after_key.find(string_open).ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No value found for key CFBundleIdentifier"))?;
    let start_of_value = string_pos + string_open.len();
    let rest_after_open = &rest_after_key[start_of_value..];

    let string_close = "</string>";
    let close_pos = rest_after_open.find(string_close).ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No value found for key CFBundleIdentifier"))?;
    let bundle_id = rest_after_open[..close_pos].trim().to_string();

    //create, download the .mobileprovision profile obtained from developer.apple
    println!("Provisioning profile for device id: {} and app bundle: {}", &uuid, &bundle_id);
     //open apple developer portal
     let output = Command::new("open")
     .args(["-a", "safari", "https://developer.apple.com/account/resources/profiles/list"])
     .stdout(Stdio::null())
     .stderr(Stdio::null())
     .output()
     .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to open Apple developer portal profile list: {}", e)))?;
     if !output.status.success() {
         return Err(io::Error::new(io::ErrorKind::Other, "Failed to open Apple developer portal profile list"));
     }    
     println!("**********************************");   
     //TODO eventually replace these prints in the GUI
     println!("1. Now go to the safari window and login to your developer account if necessary.");
     println!("2. Once you have logged in, click the + button next to \"Profiles\".");
     println!("3. Check your development or distribution options: we reccomend choosing \"{} App Development\", then press continue", target_os.as_str());
     println!("4. Select your App ID from the dropdown list: {}", &bundle_id);
     println!("5. Select whether you would like offline support (choose No if you're not sure). Then press continue.");
     println!("6. Select your Appropriate \"(Development)\" certificate. Then press continue.");

     println!("7. Select the device profile corresponding to UUID: {}", &uuid);
     println!("8. Click the continue button.");
     //TODO add support for release key here by checking release bool and changing the string
     let profile_name = if release {"Ramp Debug"} else {"Ramp Release"};
     println!("9. Enter your Provisioning profile name. Reccomended: \"{}\"", &profile_name);
     println!("10. Click \"Generate\". Then click \"Download\".");
     println!("11. Press enter to continue...");
     io::stdin().read_line(&mut input).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to receive user input: {}", e)))?;

     //detect the .mobileprovision from the Downloads folder
    let downloads_path_string = format!("{}/Downloads", &session.home);
    let downloads_path = Path::new(&downloads_path_string);
    let timeout = Duration::from_secs(60);
    let start_time = Instant::now();

    let mobileprovision_file: String;

    //loop for 60 seconds to check for a .mobileprovision in Downloads
    loop {
        if start_time.elapsed() >= timeout {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "No .mobileprovision file found within 60 seconds"));
        }

        let entries = fs::read_dir(downloads_path)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to read directory: {}", e)))?;

        let mut matching_files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to process directory entry: {}", e)))?;
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if file_name.ends_with(".mobileprovision") {
                matching_files.push(file_name);
            }
        }

        match matching_files.len() {
            0 => {
                // Continue looping
            }
            1 => {
                mobileprovision_file = matching_files[0].clone();
                break;
            }
            _ => {
                return Err(io::Error::new(io::ErrorKind::Other, "Multiple .mobileprovision files found"));
            }
        }

        // Sleep briefly
        std::thread::sleep(Duration::from_millis(500));
    }
    println!("Mobile Provision File Name: {}", &mobileprovision_file);
    let mp_origin = format!("{}/Downloads/{}", &session.home, &mobileprovision_file);
    println!("mobile provision origin path: {}", &mp_origin);
    let mp_destination = format!("{}/{}/{}/{}.app", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), &target_os, capitalize_first(session.current_project.as_ref().unwrap()));
    println!("mobile provision destination path: {}", &mp_destination);
    //cut the mobileprovision from Downloads folder to the project's app bundle
    if Path::new(&mp_origin).exists() {
        //Move the .mobileprovision to the app bundle
        let output = Command::new("mv")
            .args([&mp_origin, &mp_destination])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to move the .mobileprovision to the keychain directory"));
        }
        println!("Successfully moved the mobile provision!");
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to obtain the path to mobile provision"));
    }
    Ok(())
}

fn get_uuid_by_target(device_target: &str) -> io::Result<String> {
    // Run xcrun xctrace list devices
    let output = Command::new("xcrun")
        .args(["xctrace", "list", "devices"])
        .output()
        .map_err(|e| {
            io::Error::new(
                ErrorKind::Other,
                format!("Failed to execute xcrun xctrace: {}", e),
            )
        })?;

    if !output.status.success() {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("xcrun xctrace failed: {}", String::from_utf8_lossy(&output.stderr)),
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Define regex pattern based on device target
    let pattern = match device_target.to_lowercase().as_str() {
        "iphone" => r"(?i)^iPhone\s+\([^)]+\)\s+\(([0-9a-f]{8}-[0-9a-f]{16})\)",
        "ipad" => r"(?i)^iPad\s+\([^)]+\)\s+\(([0-9a-f]{8}-[0-9a-f]{16})\)",
        "macos" => {
            r"(?i)^[^\n]*MacBook[^\n]*\s+\(([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})\)"
        }
        _ => {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "Invalid device target: must be 'iphone', 'ipad', or 'macos'",
            ))
        }
    };

    let re = Regex::new(pattern).map_err(|e| {
        io::Error::new(ErrorKind::Other, format!("Failed to compile regex: {}", e))
    })?;

    // Collect all matching UDIDs
    let mut udids = Vec::new();
    for line in output_str.lines() {
        if let Some(captures) = re.captures(line) {
            if let Some(udid) = captures.get(1) {
                let udid_str = udid.as_str();
                // Validate UDID length (25 for iPhone/iPad, 36 for macOS)
                if (device_target.to_lowercase() == "iphone" || device_target.to_lowercase() == "ipad")
                    && udid_str.len() == 25
                    || device_target.to_lowercase() == "macos" && udid_str.len() == 36
                {
                    udids.push(udid_str.to_string());
                }
            }
        }
    }

    // Check the number of matching UDIDs
    match udids.len() {
        0 => Err(io::Error::new(
            ErrorKind::NotFound,
            format!("No {} device found", device_target),
        )),
        1 => Ok(udids[0].clone()),
        _ => Err(io::Error::new(
            ErrorKind::Other,
            format!(
                "Multiple {} devices found: {}",
                device_target,
                udids.join(", ")
            ),
        )),
    }
}

fn get_device_identifier() -> io::Result<String> {
    // Run xcrun devicectl list devices
    let output = Command::new("xcrun")
        .args(["devicectl", "list", "devices"])
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to run devicectl: {}", e)))?;

    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "devicectl command failed"));
    }

    // Parse output with regex for UUID (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}").unwrap();
    let uuids: Vec<&str> = re.find_iter(&stdout).map(|m| m.as_str()).collect();

    // Check UUID count
    match uuids.len() {
        1 => Ok(uuids[0].to_string()),
        0 => Err(io::Error::new(io::ErrorKind::NotFound, "No device identifier found")),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Multiple device IDs found")),
    }
}


fn load_simulator(session: &Session, target_os: String) -> io::Result<()>{
    println!("load_simulator");
    if target_os == "ios" {
        //TODO make sure this never tried to boot a non sim binary
        println!("deploying to {} simulator", target_os);
        //TODO check if simulator is already running first
        //open ios simuator
        let output = Command::new("open")
            .args(["-a", "simulator"])
            .output()
            .unwrap();
        if !output.status.success(){
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not open IOS simulator: {}",
            ));
        }
        //TODO create a device, need to build out support here
        //boot & install the .app bundle to the simulator
        let output = Command::new("xcrun")
            .args(["simctl", "install", "booted", &format!("{}/{}/ios/{}.app", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), capitalize_first(session.current_project.as_ref().unwrap()))])
            .output()
            .unwrap();
        
        if !output.status.success(){
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not deploy to IOS simulator: {}",
            ));
        }
    }
    //MACOS side
    //TODO macos sim
    //TODO ios sim
    //TODO android sim
    //TODO windows sim
    //TODO ubuntu sim?
    //todo wasm?

    //UBUNTU SIDE
    //TODO android sim
    //TODO windows sim
    //TODO ubuntu sim?
    //todo wasm?
    println!("finished deploying to {} simulator", target_os);
    Ok(())
}

pub fn is_device_provisioned(app_bundle_path: &str, device_id: &str) -> io::Result<bool> {
    //obtain to the mobile provision file name
    let mobileprovision_file: String;
    let entries = fs::read_dir(app_bundle_path)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to read directory: {}", e)))?;

        let mut matching_files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to process directory entry: {}", e)))?;
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if file_name.ends_with(".mobileprovision") {
                matching_files.push(file_name);
            }
        }

        match matching_files.len() {
            0 => {
                return Ok(false);
            }
            1 => {
                mobileprovision_file = matching_files[0].clone();
            }
            _ => {
                return Ok(false);
            }
        }
    let profile_path_str = format!("{}/{}", &app_bundle_path, &mobileprovision_file);
    let profile_path = Path::new(&profile_path_str);
    //query the mobile provision profile
    let output = Command::new("security")
        .arg("cms")
        .arg("-D")
        .arg("-i")
        .arg(profile_path)
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to execute security command: {}", e)))?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("security command failed: {}", String::from_utf8_lossy(&output.stderr)),
        ));
    }
    //check for an existing device provision
    let xml = String::from_utf8(output.stdout)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Invalid UTF-8 in plist: {}", e)))?;

    let key_str = "<key>ProvisionedDevices</key>";
    let Some(key_pos) = xml.find(key_str) else {
        println!("provision profile does not contain valid syntax: <key>ProvisionedDevice</key>");
        return Ok(false);
    };
    let start_after_key = key_pos + key_str.len();
    let rest_after_key = &xml[start_after_key..];

    let string_open = "<array>";
    let Some(array_pos) = rest_after_key.find(string_open) else {
        println!("provision profile does not contain valid syntax: <array>");
        return Ok(false);
    };
    let start_of_array = array_pos + string_open.len();
    let rest_after_open = &rest_after_key[start_of_array..];

    let string_close = "</array>";
    let Some(close_pos) = rest_after_open.find(string_close) else {
        println!("provision profile does not contain valid syntax: </array>");
        return Ok(false);
    };
    let array_content = &rest_after_open[..close_pos];

    let device_entry = format!("<string>{}</string>", device_id);
    if array_content.contains(&device_entry) {
        println!("target device is already provisioned");
        Ok(true)
    } else {
        println!("target device is not provisioned");
        Ok(false)
    }
}

fn deploy_usb_tether(session: &mut Session, target_os: String) -> io::Result<()> {
    //obtain device uuid
    let uuid = get_uuid_by_target("iphone")?;
    let device_id = get_device_identifier()?;
    println!("target device ID: {}", &device_id);
    //check for an existing provisioning profile
    let profile_path_str = &format!("{}/{}/{}/{}.app", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), &target_os, capitalize_first(session.current_project.as_ref().unwrap()));
    let provision_required = is_device_provisioned(&profile_path_str, &device_id)?;
    if provision_required {
        //add a new provisioning profile for a macos device
        provision_device(session, uuid, &target_os, false)?;
    }
    //deploy to target device
    if target_os == "ios"{
        println!("deploying to ios device");
        let output = Command::new("xcrun")
            .args(["devicectl", "device", "install", "app", "--device", &device_id, &format!("{}/{}/ios/{}.app", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), capitalize_first(session.current_project.as_ref().unwrap()))])
            .output()
            .unwrap();
        if !output.status.success() {
            println!("here is the output: {:?}", &output);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not install app bundle to IOS device via USB tether: {}",
            ));
        }
        let bundle_id = get_bundle_id("ios")?;
        println!("Deploying bundle id: {} to device: {}", &bundle_id, &device_id);
        let output = Command::new("xcrun")
            .args(["devicectl", "device", "process", "launch", "--device", &device_id, &bundle_id])
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not launch app bundle to IOS device via USB tether: {}",
            ));
        }
    }else if target_os == "android"{
        //TODO android device tether deployment
        println!("TODO android tether deployment");
    }

    println!("Successfully deployed to {} device", &target_os);

    Ok(())
    
}

//this needs to get called when creating a new project on macos/ios
fn create_app_bundle_id(session: &mut Session) -> io::Result<Option<String>> {
        if session.os.as_str() != "macos"{
            return Ok(None)
        }
        //open apple developer portal
        let output = Command::new("open")
        .args(["-a", "safari", "https://developer.apple.com/account/resources/identifiers/list/bundleId"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to open Apple developer portal: {}", e)))?;
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to open Apple developer portal"));
        }    

        //take in app bundle id here
        let mut bundle_id = String::new();
        println!("*********************************************");
        println!("Please enter your app bundle ID. Press enter for default (Reccomended) which will be \"com.ramp.{}\"", session.current_project.as_ref().unwrap());
        println!("*********************************************");
        io::stdin()
            .read_line(&mut bundle_id)
            .expect("failed to read line");
        let bundle_id = bundle_id.trim();
        let bundle_id = if bundle_id.is_empty() {
            format!("com.ramp.{}", session.current_project.as_ref().unwrap())
        }else {
            bundle_id.to_string()
        };
        println!("using bundle_id: {}", &bundle_id);

        println!("**********************************");   
        //TODO eventually replace these prints in the GUI
        println!("1. Now go to the safari window and login to your developer account.");
        println!("2. Once you have logged in, click the + button next to \"Identifiers\".");
        println!("3. Select \"App IDs\" and click the continue button.");
        println!("4. Select \"App\" for the type and click the continue button.");
        println!("5. Copy and paste your App Bundle ID shown below into the \"Bundle ID\" box.");
        println!("{}", &bundle_id);
        println!("6. Give your app whatever description you like and then press the continue button.");
        println!("7. Click the Register button.");
    
        if bundle_id == format!("com.ramp.{}", session.current_project.as_ref().unwrap()){
            Ok(None)
        }else {
            Ok(Some(bundle_id))
        }
}

fn build_output(session: &mut Session, target_os: String, release: bool) -> io::Result<()> {
    // Validate project path
    let project_path = format!(
        "{}/{}",
        session.projects_path.as_ref().unwrap(),
        session.current_project.as_ref().unwrap()
    );
    let project_dir = Path::new(&project_path);
    if !project_dir.exists() || !project_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Project directory not found: {}", project_path),
        ));
    }
    if !project_dir.join("Cargo.toml").exists() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("No Cargo.toml found in {}", project_path),
        ));
    }

    // Map target_os to Cargo command
    let cargo_args = match target_os.as_str() {
        "windows" => format!(
            "build --target x86_64-pc-windows-gnu{}",
            if release { " --release " } else { "" }
        ),
        "linux" => format!("build{}", if release { " --release " } else { "" }),
        "wasm" => format!(
            "build --lib --target wasm32-unknown-unknown{}",
            if release { " --release " } else { "" }
        ),
        "android" => format!(
            "apk build{}",
            if release { " --release " } else { " --lib " }
        ),
        "ios" => format!(
            "build --target aarch64-apple-ios{}",
            if release { " --release " } else { "" }
        ),
        "ios_sim" => "build --target aarch64-apple-ios-sim".to_string(),
        //TODO need to support lipo outputs for combined chipset architecture
        "macos" => format!(
            "build{}",
            if release { " --release " } else { "" }
        ),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported target OS: {}", target_os),
            ))
        }
    };

    // Execute cargo build
    let cargo_command = format!("{} {}", session.get_path("cargo_path")?, cargo_args);
    let output = if target_os.as_str() == "android" {
                    Command::new("bash")
                    .arg("-c")
                    .arg(&cargo_command)
                    .current_dir(project_dir) // Set working directory
                    //provide the environment paths for android toolchain
                    .env("JAVA_HOME", &session.get_path("java_path")?)                
                    .env("ANDROID_HOME", &session.get_path("sdk_path")?)
                    .env("NDK_HOME", &session.get_path("ndk_path")?)
                    .stdout(Stdio::inherit()) // Show build output
                    .stderr(Stdio::inherit())
                    .output()?
                } else{
                    Command::new("bash")
                    .arg("-c")
                    .arg(&cargo_command)
                    .current_dir(project_dir) // Set working directory
                    .stdout(Stdio::inherit()) // Show build output
                    .stderr(Stdio::inherit())
                    .output()?
                };
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Cargo build failed: {}", error),
        ));
    }

    println!(
        "Successfully built project at {} for target {} ({} mode)",
        project_path,
        target_os,
        if release { "release" } else { "debug" }
    );

    //post build house keeping
    if target_os == "ios" && release == false {
        println!("performing ios post build...");
        //move the binary into the ios app bundle
        let output = Command::new("cp")
        .args([&format!("{}/target/aarch64-apple-ios/debug/{}", project_path, session.current_project.as_ref().unwrap()), &format!("{}/ios/{}.app/", project_path, capitalize_first(session.current_project.as_ref().unwrap()))])
        .output()
        .unwrap();
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("ios post build failed to move binary: {}", error),
            ));
        }
        sign_build(session, target_os, release)?;
        println!("signed ios app bundle: {:?}", output);
    }else if target_os == "ios" && release == true{
        println!("TODO release build for ios");
        //TODO copy if exists /target/aarch64-apple-ios/release/appname to ios/Appname.app/
    }else if target_os == "ios_sim" {
        //TODO create a seperate app bundle for simulator?
        println!("performing ios sim post build...");
        let output = Command::new("cp")
        .args([&format!("{}/target/aarch64-apple-ios-sim/debug/{}", project_path, session.current_project.as_ref().unwrap()), &format!("{}/ios/{}.app/", project_path, capitalize_first(session.current_project.as_ref().unwrap()))])
        .output()
        .unwrap();
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("ios_sim post build failed: {}", error),
            ));
        }
    } 
    //TODO add support for all other outputs as needed
    Ok(())
}

//initialization function upon starting the app
//WARNING install must only be run with sudo privleges
fn install(session: &mut Session) -> io::Result<()> {
    //create ramp config
    create_ramp_config(&session)?;
    //get paths from any existing ramp configuration
    session.get_all_paths()?; 
    //check network connectivity
    println!("Checking for network connectivity...");
    //ping linux servers once to check for connectivity
    let output = Command::new("ping")
        .args(["-c", "1", "linux.org"])
        .output()
        .unwrap();
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "No network connection detected",
        ));
    }
    
    println!("Checking for Rust toolchain...");
    // Update rustup and cargo PATH in the ramp config
    session.set_path("cargo_path", format!("{}/.cargo/bin/cargo", session.home))?;
    session.set_path("rustup_path", format!("{}/.cargo/bin/rustup", session.home))?;
    // Check if rustup is installed
    if !is_command_available(&session.get_path("rustup_path")?) {
        println!("rustup not found. Attempting to install Rust toolchain...");
        install_rustup(session)?;
    } else {
        println!("rustup is installed.");
    }

    // Check if cargo is installed
    if !is_command_available(&session.get_path("cargo_path")?) {
        println!("cargo not found. Running rustup to ensure full toolchain...");
        install_rust_toolchain(session)?;
    } else {
        println!("cargo is installed.");
    }

    println!("Rust toolchain is ready!");

    //Install OS appropriate build targets
    install_build_targets(session)?;

    //install mac/ios toolchains
    install_macos_ios_toolchains(session)?;

    //install android toolchains
    install_android_toolchains(session)?;

    //setup keychain
    setup_keychain(session)?;

    //TODO install and configure simulators
    // install_simulators(session)?;

    //update xcode if xcode-install available
    //xcversion install --latest

    //install the needed linkers if macos and building for linux
    //brew tap messense/macos-cross-toolchains
    //brew install x86_64-unknown-linux-gnu

    //add linux linker to .cargo/config.toml if macos
    //     cat << 'EOF' > .cargo/config.toml
    // [target.x86_64-unknown-linux-gnu]
    // linker = "x86_64-unknown-linux-gnu-gcc"
    // EOF


    Ok(())
}

fn main() -> io::Result<()> {
    let mut session = Session::new()?;
    println!("Starting a new session on OS: {}", session.os);
    session.get_all_paths()?;

    // Collect all command-line arguments into a Vec<String>
    let args: Vec<String> = env::args().collect();

    // Print arguments for debugging
    println!("Arguments: {:?}", args);
    
    // Check for the -install argument, this flow requires sudo priveleges
    if args.contains(&"-install".to_string()) {
        println!("Running install with elevated privileges...");
        //initial install
        install(&mut session)?;
        //TODO move the binary from the .dmg or the .deb after install is finished
        //TODO terminate the session
        //TODO can we start an external script with a timer here to relaunch ramp gui after closing initial install client?
    }else{
        //create new proj
        let name: &str = "testproj";
        // new_project(&mut session, &name)?;
        println!("current project: {:?}", session.current_project);

        //load an existing proj
        load_project(&mut session, name)?;
        println!("current project: {:?}", session.current_project);

        // //format the icon.png in assets/resources/icons across all outputs
        // update_icons(&session)?;

        // //build the target output build_output(session: &Session, target_os: String, release: bool)
        build_output(&mut session, "ios".to_string(), false)?;

        // // load_simulator(&mut session, "ios".to_string())?;
        deploy_usb_tether(&mut session, "ios".to_string())?;
    }

    //TODOS

    //test that all app icons are properly removed and recreated after an update
    //signing keys and dev cert management
    

    //MACOS
    //set up/config key signers & dev certs
    //deploy via usb tether for ios and android
    //deploy to simulators for every build output
    //fix ubuntu output compatability (see notes in install function)
    //lipo outputs for combined chipset architectures for ios simulator and macos release

    //LINUX
    //start to finish comb through
    //ensure that all commands set paths in the .ramp config
    //rework all commands to use paths from the .ramp config
    //discard any .zsh or .bshrc persistence
    //refactor all sudo requirements outside of the -install flag, consider a .deb install script that calls sudo with an -install flag
    //setup/config key signers
    //BUILD for simulators, deploy simulator, hot load over a usb


    //WISHLIST

    //gracefully intercept and handle errors where the user's OS is out of date (particularly in the case of MacOS)

    //ability to use an existing android sdk/ndk installation

    //more robust version specification for critical components (xcode, ios ndk, jdk, android ndk & sdk, etc etc)

    //template version tracking

    //ability to customize projects path

    //ability to customize paths to binaries (can manually do this in "$HOME/.ramp" config currently)

    Ok(())
}
