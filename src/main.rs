use image::{self, imageops, DynamicImage, GenericImageView, ImageEncoder, ImageFormat};
use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::io::BufReader;
use std::io::BufRead;

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
}

struct Session {
    os: String,
    home: String,
    projects_path: Option<String>,
    current_project: Option<String>,
    paths: Paths,
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
        };
        Ok(Session {
            os,
            home,
            projects_path,
            current_project: None,
            paths,
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
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unknown path name")),
    }

    // Read existing config file
    let config_path = format!("{}/.config", self.home);
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

    Ok(())
}

// Method to read .config and update Paths struct
fn get_paths(&mut self) -> io::Result<()> {
    let config_path = format!("{}/.config", self.home);

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
                _ => (), // Ignore unknown keys
            }
        }
    }

    Ok(())
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
fn install_rustup(session: &Session) -> io::Result<()> {
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
            // Update PATH in the ramp config
            let cargo_path = format!("{}/.cargo/bin/cargo", session.home);
            session.set_path("cargo_path", cargo_path)?;
            let rustup_path = format!("{}/.cargo/bin/rustup", session.home);
            session.set_path("rustup_path", rustup_path)?;
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
            // Update PATH in the ramp config
            // Update PATH in the ramp config
            let cargo_path = format!("{}/.cargo/bin/cargo", session.home);
            session.set_path("cargo_path", cargo_path)?;
            let rustup_path = format!("{}/.cargo/bin/rustup", session.home);
            session.set_path("rustup_path", rustup_path)?;
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
fn install_rust_toolchain(session: &Session) -> io::Result<()> {
    let status = Command::new(&session.paths.rustup_path)
        .args(&["toolchain", "install", "stable"])
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to install Rust stable toolchain",
        ));
    }

    println!("Rust stable toolchain installed!");
    Ok(())
}

//install build targets for all supported ramp outputs
fn install_build_targets(session: &Session) -> io::Result<()> {
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
    let output = Command::new(&session.paths.rustup_path)
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
            let status = Command::new(&session.paths.rustup_path)
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
                let status = Command::new(&session.paths.rustup_path)
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

fn install_homebrew(session: &Session) -> io::Result<()> {
    if &session.os.as_str() != &"macos"{
        println!("skipping homebrew, not mac");
        return Ok(())
    }
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
    session.set_path("homebrew_path", &format!("{}/brew", brew_bin))?;

    Ok(())
}

fn install_macos_ios_toolchains(session: &Session) -> io::Result<()> {
    //verify that the Xcode app is already installed
    let xcode_app = "/Applications/Xcode.app";
    if !Path::new(xcode_app).exists() {
        return Err(io::Error::new(io::ErrorKind::Other, "Xcode is not properly installed via the appstore"));
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

    // Check if Homebrew is installed
    let brew_ok = Command::new(session.paths.homebrew_path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !brew_ok {
        //install homebrew if mac
        install_homebrew(&session)?;
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
            return Ok(())
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
    if !status.success() {
        eprintln!("Failed to accept Xcode license.");
        std::process::exit(1);
    }

    Ok(())
}

fn install_simulators(session: &Session) -> io::Result<()>{
    //run xcrun simctl list devices to initialize
    println!("setting up simulators");
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

fn install_android_toolchains(session: &Session) -> io::Result<()> {
    println!("Setting up Android SDK and NDK for {}", session.os);
    session.set_path("sdk_path", format!("{}/Android/sdk", session.home))?;
    session.set_path("cmdline_tools_path", format!("{}/cmdline-tools", session.paths.sdk_path))?;
    session.set_path("sdkmanager_path", format!("{}/bin/sdkmanager", session.paths.cmdline_tools_path))?;
    session.set_path("platform_tools_path", format!("{}/platform-tools/adb", session.paths.sdk_path))?;
    session.set_path("platforms_path", format!("{}/platforms/android-{}", session.paths.sdk_path, session.android_platform_version))?;
    session.set_path("ndk_path", format!("{}/ndk/{}", session.paths.sdk_path, session.android_ndk_version))?;
    session.set_path("ndk_bundle_path", format!("{}/ndk-bundle", session.paths.sdk_path))?;

    // Check for JDK
    session.set_path("java_path", "/opt/homebrew/opt/openjdk@17/bin/java")?;
    println!("Java path: {}", session.paths.java_path);
    let java_ok = match Command::new(&session.paths.java_path)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output() {
            Ok(output) => String::from_utf8_lossy(&output.stderr)
                .to_lowercase()
                .contains("openjdk"),
            Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "failed to query java version")),
        };
 

    // OS-specific configuration
    let (java_home, shell_config, sdk_url, install_jdk): (
        &str,
        String,
        &str,
        Box<dyn Fn() -> io::Result<()>>,
    ) = match session.os.as_str() {
        "linux" => {
            let java_home = session.paths.java_path;
            println!("Java home: {}", java_home.to_string());
            (
                "/usr/lib/jvm/java-17-openjdk-amd64",
                format!("{}/.bashrc", session.home),
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
            (
                java_home,
                format!("{}/.zshrc", session.home),
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

        let sdkmanager_ok = Path::new(&session.paths.sdkmanager_path).exists();
        let platform_tools_ok = Path::new(&session.paths.platform_tools_path).exists()
            && Command::new(&session.paths.platform_tools_path)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        let platforms_ok = Path::new(&session.paths.platforms_path).exists();
        let ndk_ok = Path::new(&session.paths.ndk_path).exists() || Path::new(&session.paths.ndk_bundle_path).exists();
        if !ndk_ok {
            let ndk_dir = format!("{}/ndk", session.paths.sdk_path);
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
            sdkmanager_ok, session.paths.sdkmanager_path,
            platform_tools_ok, session.paths.platform_tools_path,
            platforms_ok, session.paths.platforms_path,
            ndk_ok, session.paths.ndk_path
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
            .args(&["-p", &session.paths.sdk_path])
            .status()?;
        Command::new("unzip")
            .args(&["-o", &download_path, "-d", &session.paths.sdk_path])
            .status()?;
        Command::new("rm")
            .arg(&download_path)
            .status()?;
        // Accept licenses
        println!("Accepting Android SDK licenses...");
        let mut license_cmd = Command::new("yes")
            .stdout(Stdio::piped())
            .spawn()?;
        let license_output = Command::new(&session.paths.sdkmanager_path)
            .args(["--licenses", &format!("--sdk_root={}", &session.paths.sdk_path)])
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
            let install_output = Command::new(&session.paths.sdkmanager_path)
                .args(&[package, &format!("--sdk_root={}", &session.paths.sdk_path)])
                .output()?;
            if !install_output.status.success() {
                println!("Install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
                return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to install {}", package)));
            }
        }
        println!("Android SDK and NDK installed.");
    } else {
        println!("Existing Android SDK and NDK found at {}. Skipping installation.", session.paths.sdk_path);
    }

    // Set environment variables for current process
    //TODO can we remove this safely without breaking android dependencies?
    env::set_var("ANDROID_HOME", &session.paths.sdk_path);
    let ndk_home = if Path::new(&session.paths.ndk_path).exists() { &session.paths.ndk_path } else { &session.paths.ndk_bundle_path };
    //TODO can we remove this safely without breaking android dependencies?
    env::set_var("NDK_HOME", ndk_home);
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}/platform-tools:{}", current_path, session.paths.sdk_path, ndk_home);
    //TODO can we remove this safely without breaking android dependencies?
    env::set_var("PATH", &new_path);
    println!(
        "Set environment for current session: JAVA_HOME={}, ANDROID_HOME={}, NDK_HOME={}, PATH={}",
        java_home, session.paths.sdk_path, ndk_home, new_path
    );

    // Persist environment variables in shell config
    //TODO can we remove this safely without breaking android dependencies?
    let env_entries = format!(
        "\nexport JAVA_HOME={}\nexport ANDROID_HOME={}\nexport NDK_HOME={}\nexport PATH=$PATH:{}/platform-tools:{}\n",
        java_home, session.paths.sdk_path, ndk_home, session.paths.sdk_path, ndk_home
    );
    let mut shell_content = if Path::new(&shell_config).exists() {
        fs::read_to_string(&shell_config)?
    } else {
        String::new()
    };
    if !shell_content.contains(&env_entries) {
        let mut shell_file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&shell_config)?;
        shell_file.write_all(env_entries.as_bytes())?;
        println!("Added JAVA_HOME, ANDROID_HOME, NDK_HOME, and PATH to {}", shell_config);
    } else {
        println!("Environment variables already in {}", shell_config);
    }

    // Install cargo-apk
    session.set_path("cargo_apk_path", format!("{}/.cargo/bin/cargo-apk", session.home))?;
    let cargo_apk_ok = Command::new(&session.paths.cargo_apk_path)
        .args(&["apk", "version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if cargo_apk_ok {
        println!("cargo-apk is already installed. Skipping installation.");
    } else {
        // Ensure Android SDK/NDK and JAVA_HOME are set
        let android_home = env::var("ANDROID_HOME").unwrap_or_default();
        let ndk_home = env::var("NDK_HOME").unwrap_or_default();
        let java_home = env::var("JAVA_HOME").unwrap_or_default();
        if android_home.is_empty() || ndk_home.is_empty() || java_home.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "ANDROID_HOME, NDK_HOME, or JAVA_HOME not set.",
            ));
        }
        // Install cargo-apk
        println!("Installing cargo-apk...");
        let install_output = Command::new("cargo")
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

    //rename everything inside of the template with the project name
    template_naming(session, &name.to_lowercase())?;

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
fn template_naming(session: &mut Session, name: &str) -> io::Result<()> {
    let new_path = format!(
        "{}/{}",
        session.projects_path.as_ref().unwrap_or(&String::new()),
        name
    );
    let capitalized_name = capitalize_first(name);
    //rename app in Cargo.toml
    let replacements = vec![("Webgpu", capitalized_name.as_str()), ("webgpu", name)];
    replace_strings_in_file(&format!("{}/Cargo.toml", new_path), &replacements)?;
    //rename dir ios/Webgpu.app
    rename_directory(
        &format!("{}/ios/Webgpu.app", new_path),
        &format!("{}.app", &capitalized_name),
    )?;
    //rename ios/Webgpu.app/Info.plist
    replace_strings_in_file(
        &format!("{}/ios/{}.app/Info.plist", new_path, capitalized_name),
        &replacements,
    )?;
    //rename dir macos/Webgpu.app
    rename_directory(
        &format!("{}/macos/Webgpu.app", new_path),
        &format!("{}.app", &capitalized_name),
    )?;
    //rename macos/Webgpu.app/Contents/Info.plist
    replace_strings_in_file(
        &format!("{}/ios/{}.app/Info.plist", new_path, capitalized_name),
        &replacements,
    )?;
    //rename Cargo.toml internals
    let replacements = vec![("webgpu", name), ("ramp_template", name)];
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

    //TODO macos only
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

// fn load_simulator(session: &Session, target_os: String, release: bool) -> io::Result<()>{
//     println!("TODO load_simulator");
//     //MACOS side
//     //TODO macos sim
//     //TODO ios sim
//     //TODO android sim
//     //TODO windows sim
//     //TODO ubuntu sim?
//     //todo wasm?

//     //UBUNTU SIDE
//     //TODO android sim
//     //TODO windows sim
//     //TODO ubuntu sim?
//     //todo wasm?
// }

fn build_output(session: &Session, target_os: String, release: bool) -> io::Result<()> {
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
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("{} {}", session.paths.cargo_path, cargo_args))
        .current_dir(project_dir) // Set working directory
        .stdout(Stdio::inherit()) // Show build output
        .stderr(Stdio::inherit())
        .output()?;
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
    Ok(())
}

//initialization function upon starting the app
//WARNING install must only be run with sudo privleges
fn install(session: &Session) -> io::Result<()> {
    //create ramp config
    create_ramp_config(&session)?;

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
    // Check if rustup is installed
    if !is_command_available("rustup") {
        println!("rustup not found. Attempting to install Rust toolchain...");
        install_rustup(&session)?;
    } else {
        println!("rustup is installed.");
    }

    // Check if cargo is installed
    if !is_command_available(session.paths.cargo_path) {
        println!("cargo not found. Running rustup to ensure full toolchain...");
        install_rust_toolchain()?;
    } else {
        println!("cargo is installed.");
    }

    println!("Rust toolchain is ready!");

    //Install OS appropriate build targets
    install_build_targets(&session)?;

    //install mac/ios toolchains
    install_macos_ios_toolchains(&session)?;

    //install android toolchains
    install_android_toolchains(&session)?;

    //TODO install and configure simulators
    install_simulators(&session)?;

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

    //TODO verify that both zsh and bsh are supported across implementations

    Ok(())
}

fn main() -> io::Result<()> {
    let mut session = Session::new()?;
    println!("Starting a new session on OS: {}", session.os);
    session.get_paths()?;

    // Collect all command-line arguments into a Vec<String>
    let args: Vec<String> = env::args().collect();

    // Print arguments for debugging
    println!("Arguments: {:?}", args);
    
    // Check for the -installation argument, this flow requires sudo priveleges
    if args.contains(&"-installation".to_string()) {
        println!("Running installation with elevated privileges...");
        //initial install
        install(&session)?;
        //TODO terminate the session here
        //TODO move the binary from the .dmg or the .deb after install is finished
    }else{
        //create new proj
        let name: &str = "testproj";
        // new_project(&mut session, &name)?;
        println!("current project: {:?}", session.current_project);

        //load an existing proj
        load_project(&mut session, name)?;
        println!("current project: {:?}", session.current_project);

        //format the icon.png in assets/resources/icons across all outputs
        update_icons(&session)?;

        //build the target output build_output(session: &Session, target_os: String, release: bool)
        build_output(&session, "ios".to_string(), false)?;
    }

    //TODOS

    //ADD A /Users/$USER/.ramp global config to set the paths for all binaries required for ramp
    //Remove all .zshrc configurations
    //use full paths for all binary command executions


    //test that all app icons are properly removed and recreated after an update
    //release key and dev cert management
    //deploy to simulators

    //MACOS
    //fix ubuntu output compatability (see notes in install function)
    //set up/config key signers
    //lipo outputs for combined chipset architectures for ios simulator and macos release

    //LINUX
    //refactor all sudo requirements outside of the -installation flag, consider a .deb install script that calls sudo with an -installation flag
    //setup/config key signers
    //BUILD for simulators, deploy simulator, hot load over a usb


    //WISHLIST

    //gracefully intercept and handle errors where the user's OS is out of date (particularly in the case of MacOS)

    //ability to use an existing android sdk/ndk installation

    //more robust version specification for critical components (xcode, ios ndk, jdk, android ndk & sdk, etc etc)

    //template version tracking

    //ability to customize projects path

    Ok(())
}
