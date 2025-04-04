use std::process::{Command, Stdio};
use std::io::{self, Write};
use std::env;
use std::path::Path;


struct Session {
    os: String,
    projects_path: Option<String>,
}

impl Session {
    fn new() -> Self {
        Session {
            os: env::consts::OS.to_string(),
            projects_path: None,
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

    match session.os.as_str() {
        "linux" => {
            //update apt
            println!("running sudo apt update...");
            let status = Command::new("sudo")
                .args(["apt", "update"])
                .status()?;
            if !status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to apt update",
                ));
            }
            //install curl
            println!("installing curl...");
            let status = Command::new("sudo")
                .args(["apt", "install", "curl", "-y"])
                .status()?;
            if !status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to install curl",
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
            let home = env::var("HOME").map_err(|e| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Failed to get HOME environment variable: {}", e),
                )
            })?;
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
            // Update PATH for the current session
            let home = env::var("HOME").map_err(|e| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Failed to get HOME environment variable: {}", e),
                )
            })?;
            let current_path = env::var("PATH").unwrap_or_default();
            env::set_var("PATH", format!("{}/.cargo/bin:{}", home, current_path));
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
            "Failed to list installed rustup targets"
        ))
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
        } else{
            println!("Target: {} already installed", target);
        }
    }

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
            } else{
                println!("Target: {} already installed", target);
            }
        }
    }
    println!("Build targets installed!");
    Ok(())
}

fn install_android_sdk_and_ndk(session: &Session) -> io::Result<()> {
    println!("Setting up Android SDK and NDK for {}", session.os);

    let home = env::var("HOME").map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Failed to get HOME: {}", e)))?;
    let sdk_root = format!("{}/Android/sdk", home);
    let cmdline_tools_dir = format!("{}/cmdline-tools", sdk_root);
    let desired_ndk_version = "26.1.10909125";

    // Check for existing SDK
    let mut sdkmanager_path = None;
    let possible_sdk_locations = vec![
        env::var("ANDROID_HOME").ok(),
        env::var("ANDROID_SDK_ROOT").ok(),
        Some(sdk_root.clone()),
    ];
    for location in possible_sdk_locations.into_iter().flatten() {
        let candidate = format!("{}/cmdline-tools/latest/bin/sdkmanager", location);
        if Path::new(&candidate).exists() {
            let status = Command::new(&candidate)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()?;
            if status.success() {
                sdkmanager_path = Some(candidate);
                println!("Found existing Android SDK at {}", location);
                break;
            }
        }
    }

    // Install SDK if not found
    if sdkmanager_path.is_none() {
        println!("Android SDK not found. Installing command-line tools...");
        match session.os.as_str() {
            "linux" | "macos" => {
                let sdk_url = "https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip";
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
                Command::new("mv")
                    .args(&[format!("{}/cmdline-tools", sdk_root), cmdline_tools_dir.clone()])
                    .status()?;
                Command::new("rm")
                    .arg(&download_path)
                    .status()?;
                sdkmanager_path = Some(format!("{}/latest/bin/sdkmanager", cmdline_tools_dir));
            }
            _ => return Err(io::Error::new(io::ErrorKind::Other, "Unsupported OS for Android SDK installation")),
        }
        println!("Android command-line tools installed.");
    }

    let sdkmanager = sdkmanager_path.unwrap();

    // Check for NDK
    let ndk_path = format!("{}/ndk/{}", sdk_root, desired_ndk_version);
    let ndk_installed = Path::new(&ndk_path).exists() || Path::new(&format!("{}/ndk-bundle", sdk_root)).exists();

    // Accept licenses and install packages if needed
    if !ndk_installed {
        println!("Accepting Android SDK licenses...");
        let status = Command::new(&sdkmanager)
            .args(&["--licenses", "--no-ui"])
            .status()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to accept Android SDK licenses"));
        }
        let ndk_package = format!("ndk;{}", desired_ndk_version);
        let packages = if !ndk_installed {
            vec!["platform-tools", "build-tools;34.0.0", &ndk_package]
        } else {
            vec![]
        };

        for package in packages {
            println!("Installing {}...", package);
            let status = Command::new(&sdkmanager)
                .args(&[package])
                .status()?;
            if !status.success() {
                return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to install {}", package)));
            }
        }
    } else {
        println!("Android NDK version {} already installed at {}", desired_ndk_version, ndk_path);
    }

    // Update PATH
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!(
        "{}:{}/platform-tools:{}/ndk/{}",
        current_path, sdk_root, sdk_root, desired_ndk_version
    );
    env::set_var("PATH", &new_path);
    println!("PATH updated:\n{}", new_path);

    Ok(())
}


fn startup(session: &Session) -> io::Result<()> {
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

    install_android_sdk_and_ndk(&session)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let session = Session::new();
    println!("Starting a new session on OS: {}", session.os);
    startup(&session);
    //TODOS

    //Install android SDK & NDK

    //install_android_tools

    //install homebrew if mac (need to test thoroughly)
    ///bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    //install xcode if mac
    //ruby -v
    //if ruby not installed
    //brew install ruby
    //gem install xcode-install

    //update xcode if xcode-install available
    //xcversion install --latest

    //install xcode tools if mac
    //sudo rm -rf /Library/Developer/CommandLineTools
    //sudo xcode-select --install
    //softwareupdate --install --all

    //accept xcode license
    //sudo xcodebuild -license accept

    //install the needed linkers if macos
    //brew tap messense/macos-cross-toolchains
    //brew install x86_64-unknown-linux-gnu

    //add linux linker to .cargo/config.toml if macos
    //     cat << 'EOF' > .cargo/config.toml
    // [target.x86_64-unknown-linux-gnu]
    // linker = "x86_64-unknown-linux-gnu-gcc"
    // EOF

    //install all other dependencies needed based on OS (might require homebrew first for macos)


    //set projects path & save custom path with a conf

    //check project for a .ramp

    //create a new template from github template ramp_template

    //load and existing project

    //set up key signers for android & ios based on OS

    //Single Icon depository with global configuration

    //BUILD for target environments

    //BUILD for simulators

    Ok(())
}


