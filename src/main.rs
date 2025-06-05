use image::{self, imageops, DynamicImage, GenericImageView, ImageEncoder, ImageFormat};
use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::fs::OpenOptions;
use std::path::PathBuf;

struct Session {
    os: String,
    projects_path: Option<String>,
    current_project: Option<String>,
}

impl Session {
    fn new() -> io::Result<Self> {
        let os = env::consts::OS.to_string();
        let projects_path = match os.as_str() {
            "linux" => {
                let home = get_user_home()?;
                Some(format!("{}/ramp", home))
            }
            "macos" => {
                let home = get_user_home()?;
                Some(format!("{}/ramp", home))
            },
            //unsupported OS
            _ => None,
        };
        Ok(Session {
            os,
            projects_path,
            current_project: None,
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
    let home = get_user_home()?;
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
            // Update PATH for the current session
            let current_path = env::var("PATH").unwrap_or_default();
            env::set_var("PATH", format!("{}/.cargo/bin:{}", home, current_path));
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
            let permissions = Command::new("sudo").args(["chown", "-R", &sudo_user, &format!("{}/.cargo", &home)]).output()?;
            if !permissions.status.success(){
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to enable permissions for .cargo directory"));
            }
            // Update PATH for the current session
            let current_path = env::var("PATH").unwrap_or_default();
            env::set_var("PATH", format!("{}/.cargo/bin:{}", home, current_path));
            // Persist cargo PATH in .bashrc
            let cargo_bin = format!("{}/.cargo/bin", home);
            let bashrc_path = format!("{}/.zshrc", home);
            let path_entry = format!("\nexport PATH=$PATH:{}\n", cargo_bin);
            let mut bashrc_content = if Path::new(&bashrc_path).exists() {
                fs::read_to_string(&bashrc_path)?
            } else {
                String::new()
            };
            if !bashrc_content.contains(&path_entry) {
                let mut bashrc_file = fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(&bashrc_path)?;
                bashrc_file.write_all(path_entry.as_bytes())?;
                println!("Added cargo bin PATH to {}", bashrc_path);
            } else {
                println!("cargo bin PATH already in {}", bashrc_path);
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
fn install_rust_toolchain() -> io::Result<()> {
    let status = Command::new("rustup")
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
    let output = Command::new("rustup")
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
            let status = Command::new("rustup")
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
                let status = Command::new("rustup")
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
    let home = get_user_home()?;
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
    // Set PATH for current session
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", current_path, brew_bin);
    env::set_var("PATH", &new_path);
    println!("Set PATH for current session: {}", new_path);

    // Persist PATH in ~/.zshrc
    let zshrc_path = format!("{}/.zshrc", home);
    let path_entry = format!("\nexport PATH=$PATH:{}\n", brew_bin);
    let mut zshrc_content = if Path::new(&zshrc_path).exists() {
        fs::read_to_string(&zshrc_path)?
    } else {
        String::new()
    };
    if !zshrc_content.contains(&path_entry) {
        let mut zshrc_file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&zshrc_path)?;
        zshrc_file.write_all(path_entry.as_bytes())?;
        println!("Added Homebrew PATH to {}", zshrc_path);
    } else {
        println!("Homebrew PATH already in {}", zshrc_path);
    }

    Ok(())
}

fn install_macos_ios_toolchains(session: &Session) -> io::Result<()> {
    if session.os.as_str() != "macos"{
        println!("skipping macos & ios toolchain install");
    }else{
        println!("installing ios and macos toolchains");
    }
    // Determine Homebrew path based on architecture
    let brew_path = if cfg!(target_arch = "aarch64") {
        "/opt/homebrew/bin"
    } else {
        "/usr/local/bin"
    };

    // Check if Homebrew is installed
    let brew_ok = Command::new("brew")
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
    //add xcode command line tools to path


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

fn is_xcode_tools_installed() -> bool {
    Command::new("xcode-select")
        .arg("-p")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}


//install android dev kits
// fn install_android_toolchains(session: &Session) -> io::Result<()> {
//     println!("Setting up Android SDK and NDK for {}", session.os);
//     let home = env::var("HOME").map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Failed to get HOME: {}", e)))?;
//     let sdk_root = format!("{}/Android/sdk", home);
//     let cmdline_tools_dir = format!("{}/cmdline-tools", sdk_root);
//     let desired_ndk_version = "26.1.10909125";
//     let sdkmanager = format!("{}/bin/sdkmanager", cmdline_tools_dir);
//     let platform_tools = format!("{}/platform-tools/adb", sdk_root);
//     let platform_version = "31"; //API 31 (Android 14)
//     let platforms_dir = format!("{}/platforms/android-{}", sdk_root, platform_version);
//     let ndk_path = format!("{}/ndk/{}", sdk_root, desired_ndk_version);

//     // Check for JAVA_HOME and JDK
//     let java_home = env::var("JAVA_HOME").unwrap_or_default();
//     let java_ok = Command::new("java")
//         .arg("-version")
//         .stdout(Stdio::null())
//         .stderr(Stdio::null())
//         .status()
//         .map(|s| s.success())
//         .unwrap_or(false);
//     if !java_ok || java_home.is_empty() {
//         println!("JDK not found or JAVA_HOME not set. Installing OpenJDK 17...");
//         println!("java_home: {}", &java_home.to_string());
//         println!("java_ok: {}", &java_ok.to_string());
//         let update_output = Command::new("sudo")
//             .args(&["bash", "-c", "apt update"])
//             .output()?;
//         if !update_output.status.success() {
//             println!("apt update stderr: {}", String::from_utf8_lossy(&update_output.stderr));
//             return Err(io::Error::new(io::ErrorKind::Other, "Failed to run apt update"));
//         }
//         let install_output = Command::new("sudo")
//             .args(&["bash", "-c", "apt install -y openjdk-17-jdk"])
//             .output()?;
//         if !install_output.status.success() {
//             println!("apt install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
//             return Err(io::Error::new(io::ErrorKind::Other, "Failed to install OpenJDK 17"));
//         }
//         // Set JAVA_HOME for current process
//         let java_home = "/usr/lib/jvm/java-17-openjdk-amd64";
//         env::set_var("JAVA_HOME", java_home);
//         println!("Installed OpenJDK 17 and set JAVA_HOME={}", java_home);
//     } else {
//         println!("JDK found with JAVA_HOME={}", java_home);
//     }

//     // Check for existing SDK/NDK configuration
//     let sdk_configured = {
//         let sdkmanager_ok = Path::new(&sdkmanager).exists()
//             && Command::new(&sdkmanager)
//                 .args([&format!("--sdk_root={}", &sdk_root),"--version"])
//                 .stdout(Stdio::null())
//                 .stderr(Stdio::null())
//                 .status()
//                 .map(|s| s.success())
//                 .unwrap_or(false);
//         let platform_tools_ok = Path::new(&platform_tools).exists()
//             && Command::new("adb")
//                 .arg("--version")
//                 .stdout(Stdio::null())
//                 .stderr(Stdio::null())
//                 .status()
//                 .map(|s| s.success())
//                 .unwrap_or(false);
//         let platforms_ok = Path::new(&platforms_dir).exists();
//         let ndk_ok = Path::new(&ndk_path).exists();
//         if !ndk_ok {
//             let ndk_dir = format!("{}/ndk", sdk_root);
//             if Path::new(&ndk_dir).exists() {
//                 let versions = fs::read_dir(&ndk_dir)?
//                     .filter_map(|entry| entry.ok())
//                     .filter(|entry| entry.path().is_dir())
//                     .map(|entry| entry.file_name().into_string().unwrap_or_default())
//                     .collect::<Vec<_>>();
//                 println!("Available NDK versions: {:?}", versions);
//             }
//         }
//         println!(
//             "SDK checks: sdkmanager={} ({}), platform_tools={} ({}), platforms={} ({}) ndk={} ({})",
//             sdkmanager_ok, sdkmanager,
//             platform_tools_ok, platform_tools,
//             platforms_ok, platforms_dir,
//             ndk_ok, ndk_path
//         );
//         sdkmanager_ok && platform_tools_ok && platforms_ok && ndk_ok
//     };

//     if sdk_configured {
//         println!("Existing Android SDK and NDK found at {}. Skipping installation.", sdk_root);
//     } else {
//         // Proceed with installation if not configured
//         // Download and install command-line tools
//         println!("Installing Android command-line tools...");
//         let sdk_url = "https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip";
//         let download_path = format!("{}/cmdline-tools.zip", home);
//         Command::new("curl")
//             .args(&["-o", &download_path, sdk_url])
//             .status()?;
//         Command::new("mkdir")
//             .args(&["-p", &sdk_root])
//             .status()?;
//         Command::new("unzip")
//             .args(&["-o", &download_path, "-d", &sdk_root])
//             .status()?;
//         Command::new("rm")
//             .arg(&download_path)
//             .status()?;
//         // Accept licenses
//         println!("Accepting Android SDK licenses...");
//         let mut license_cmd = Command::new("yes")
//             .stdout(Stdio::piped())
//             .spawn()?;
//         let license_output = Command::new(&sdkmanager)
//             .args(["--licenses", &format!("--sdk_root={}", &sdk_root)])
//             .stdin(license_cmd.stdout.take().unwrap())
//             .output()?;
//         license_cmd.wait()?;
//         if !license_output.status.success() {
//             println!("License acceptance stderr: {}", String::from_utf8_lossy(&license_output.stderr));
//             return Err(io::Error::new(io::ErrorKind::Other, "Failed to accept Android SDK licenses"));
//         }
//         // Install SDK and NDK packages
//         let ndk_package = format!("ndk;{}", desired_ndk_version);
//         let platform_package = format!("platforms;android-{}", platform_version);
//         let packages = vec!["platform-tools", "build-tools;34.0.0", &platform_package, &ndk_package];
//         for package in packages {
//             println!("Installing {}...", package);
//             let install_output = Command::new(&sdkmanager)
//                 .args(&[package, &format!("--sdk_root={}", &sdk_root)])
//                 .output()?;
//             if !install_output.status.success() {
//                 println!("Install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
//                 return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to install {}", package)));
//             }
//         }
//         println!("Android SDK and NDK installed.");
//     }

//     // Set environment variables for current process
//     env::set_var("JAVA_HOME", "/usr/lib/jvm/java-17-openjdk-amd64");
//     env::set_var("ANDROID_HOME", &sdk_root);
//     env::set_var("NDK_HOME", &ndk_path);
//     let current_path = env::var("PATH").unwrap_or_default();
//     let new_path = format!(
//         "{}:{}/platform-tools:{}", // Removed ndk from PATH as NDK_HOME is used
//         current_path, sdk_root, &ndk_path
//     );
//     env::set_var("PATH", &new_path);
//     println!("Set environment for current session: JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64, ANDROID_HOME={}, NDK_HOME={}, PATH={}", sdk_root, &ndk_path, new_path);

//     // Persist environment variables in .bashrc
//     let bashrc_path = format!("{}/.bashrc", home);
//     let env_entries = format!(
//         "\nexport JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64\nexport ANDROID_HOME={}\nexport NDK_HOME={}\nexport PATH=$PATH:{}/platform-tools:{}\n",
//         sdk_root, &ndk_path, sdk_root, &ndk_path
//     );
//     let mut bashrc_content = if Path::new(&bashrc_path).exists() {
//         fs::read_to_string(&bashrc_path)?
//     } else {
//         String::new()
//     };
//     if !bashrc_content.contains(&env_entries) {
//         let mut bashrc_file = fs::OpenOptions::new()
//             .write(true)
//             .append(true)
//             .create(true)
//             .open(&bashrc_path)?;
//         bashrc_file.write_all(env_entries.as_bytes())?;
//         println!("Added JAVA_HOME, ANDROID_HOME, NDK_HOME, and PATH to {}", bashrc_path);
//     } else {
//         println!("Environment variables already in {}", bashrc_path);
//     }

//         // Check if cargo-apk is already installed
//         let cargo_apk_ok = Command::new("cargo")
//         .args(&["apk", "--version"])
//         .stdout(Stdio::null())
//         .stderr(Stdio::null())
//         .status()
//         .map(|s| s.success())
//         .unwrap_or(false);
//     if cargo_apk_ok {
//         println!("cargo-apk is already installed. Skipping installation.");
//     } else {
//         // Ensure Android SDK/NDK and JAVA_HOME are set
//         let android_home = env::var("ANDROID_HOME").unwrap_or_default();
//         let ndk_home = env::var("NDK_HOME").unwrap_or_default();
//         let java_home = env::var("JAVA_HOME").unwrap_or_default();
//         if android_home.is_empty() || ndk_home.is_empty() || java_home.is_empty() {
//             return Err(io::Error::new(
//                 io::ErrorKind::NotFound,
//                 "ANDROID_HOME, NDK_HOME, or JAVA_HOME not set. Please run install_android_sdk_and_ndk first.",
//             ));
//         }

//         // Install cargo-apk
//         println!("Installing cargo-apk...");
//         let install_output = Command::new("cargo")
//             .args(&["install", "cargo-apk"])
//             .stdout(Stdio::inherit())
//             .stderr(Stdio::inherit())
//             .output()?;
//         if !install_output.status.success() {
//             println!("cargo install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
//             return Err(io::Error::new(io::ErrorKind::Other, "Failed to install cargo-apk"));
//         }
//         println!("cargo-apk installed successfully.");
//     }

//     Ok(())
// }

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
    let home = get_user_home()?;
    let sdk_root = format!("{}/Android/sdk", home);
    let cmdline_tools_dir = format!("{}/cmdline-tools", sdk_root);
    let desired_ndk_version = "26.1.10909125";
    let platform_version = "31"; // API 31 (Android 12)
    let sdkmanager = format!("{}/bin/sdkmanager", cmdline_tools_dir);
    let platform_tools = format!("{}/platform-tools/adb", sdk_root);
    let platforms_dir = format!("{}/platforms/android-{}", sdk_root, platform_version);
    let ndk_path = format!("{}/ndk/{}", sdk_root, desired_ndk_version);
    let ndk_bundle = format!("{}/ndk-bundle", sdk_root);

    // Check for JAVA_HOME and JDK
    let java_home = env::var("JAVA_HOME").unwrap_or_default();
    let java_ok = Command::new("java")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    // OS-specific configuration
    let (java_home, shell_config, sdk_url, install_jdk): (
        &str,
        String,
        &str,
        Box<dyn Fn() -> io::Result<()>>,
    ) = match session.os.as_str() {
        "linux" => (
            "/usr/lib/jvm/java-17-openjdk-amd64",
            format!("{}/.bashrc", home),
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
        ),
        "macos" => {
            let sudo_user = env::var("SUDO_USER").map_err(|_| io::Error::new(io::ErrorKind::NotFound, "SUDO USER NOT FOUND"))?;
            let java_home = if cfg!(target_arch = "aarch64") {
                "/opt/homebrew/opt/openjdk@17"
            } else {
                "/usr/local/opt/openjdk@17"
            };
            (
                java_home,
                format!("{}/.zshrc", home),
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
        // "macos" => (
        //     sudo_user = env::var("SUDO_USER").map_err(|_| io::Error::new(ErrorKind::NotFound, "SUDO USER NOT FOUND"))?;
        //     if cfg!(target_arch = "aarch64") {
        //         "/opt/homebrew/opt/openjdk@17"
        //     } else {
        //         "/usr/local/opt/openjdk@17"
        //     },
        //     format!("{}/.zshrc", home),
        //     "https://dl.google.com/android/repository/commandlinetools-mac-11076708_latest.zip",
        //     Box::new(|| -> io::Result<()> {
        //         println!("Installing OpenJDK 17...");
        //         let install_output = Command::new("su")
        //             .args(&[&sudo_user, "-c", "install", "openjdk@17"])
        //             .output()?;
        //         if !install_output.status.success() {
        //             println!("brew install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
        //             return Err(io::Error::new(io::ErrorKind::Other, "Failed to install OpenJDK 17"));
        //         }
        //         Ok(())
        //     }),
        // ),
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
        env::set_var("JAVA_HOME", java_home);
        println!("Installed OpenJDK 17 and set JAVA_HOME={}", java_home);
    } else {
        println!("JDK found with JAVA_HOME={}", java_home);
    }

    // Check for existing SDK/NDK configuration
    let sdk_configured = {
        let sdkmanager_ok = Path::new(&sdkmanager).exists()
            && Command::new(&sdkmanager)
                .args([&format!("--sdk_root={}", &sdk_root), "--version"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        let platform_tools_ok = Path::new(&platform_tools).exists()
            && Command::new("adb")
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        let platforms_ok = Path::new(&platforms_dir).exists();
        let ndk_ok = Path::new(&ndk_path).exists() || Path::new(&ndk_bundle).exists();
        if !ndk_ok {
            let ndk_dir = format!("{}/ndk", sdk_root);
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
            sdkmanager_ok, sdkmanager,
            platform_tools_ok, platform_tools,
            platforms_ok, platforms_dir,
            ndk_ok, ndk_path
        );
        sdkmanager_ok && platform_tools_ok && platforms_ok && ndk_ok
    };

    if !sdk_configured {
        // Download and install command-line tools
        println!("Installing Android command-line tools...");
        let download_path = format!("{}/cmdline-tools.zip", home);
        Command::new("curl")
            .args(&["-o", &download_path, sdk_url])
            .status()?;
        Command::new("mkdir")
            .args(&["-p", &sdk_root])
            .status()?;
        Command::new("unzip")
            .args(&["-o", &download_path, "-d", &sdk_root])
            .status()?;
        Command::new("rm")
            .arg(&download_path)
            .status()?;
        // Accept licenses
        println!("Accepting Android SDK licenses...");
        let mut license_cmd = Command::new("yes")
            .stdout(Stdio::piped())
            .spawn()?;
        let license_output = Command::new(&sdkmanager)
            .args(["--licenses", &format!("--sdk_root={}", &sdk_root)])
            .stdin(license_cmd.stdout.take().unwrap())
            .output()?;
        license_cmd.wait()?;
        if !license_output.status.success() {
            println!("License acceptance stderr: {}", String::from_utf8_lossy(&license_output.stderr));
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to accept Android SDK licenses"));
        }
        // Install SDK and NDK packages
        let ndk_package = format!("ndk;{}", desired_ndk_version);
        let platform_package = format!("platforms;android-{}", platform_version);
        let packages = vec!["platform-tools", "build-tools;34.0.0", &platform_package, &ndk_package];
        for package in packages {
            println!("Installing {}...", package);
            let install_output = Command::new(&sdkmanager)
                .args(&[package, &format!("--sdk_root={}", &sdk_root)])
                .output()?;
            if !install_output.status.success() {
                println!("Install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
                return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to install {}", package)));
            }
        }
        println!("Android SDK and NDK installed.");
    } else {
        println!("Existing Android SDK and NDK found at {}. Skipping installation.", sdk_root);
    }

    // Set environment variables for current process
    env::set_var("JAVA_HOME", java_home);
    env::set_var("ANDROID_HOME", &sdk_root);
    let ndk_home = if Path::new(&ndk_path).exists() { &ndk_path } else { &ndk_bundle };
    env::set_var("NDK_HOME", ndk_home);
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}/platform-tools:{}", current_path, sdk_root, ndk_home);
    env::set_var("PATH", &new_path);
    println!(
        "Set environment for current session: JAVA_HOME={}, ANDROID_HOME={}, NDK_HOME={}, PATH={}",
        java_home, sdk_root, ndk_home, new_path
    );

    // Persist environment variables in shell config
    let env_entries = format!(
        "\nexport JAVA_HOME={}\nexport ANDROID_HOME={}\nexport NDK_HOME={}\nexport PATH=$PATH:{}/platform-tools:{}\n",
        java_home, sdk_root, ndk_home, sdk_root, ndk_home
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
    let cargo_apk_ok = Command::new("cargo")
        .args(&["apk", "--version"])
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
        .arg(format!("cargo {}", cargo_args))
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
    if !is_command_available("cargo") {
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
