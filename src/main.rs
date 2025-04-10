use std::process::{Command, Stdio};
use std::io::{self, Write};
use std::env;
use std::path::Path;
use std::fs;



struct Session {
    os: String,
    projects_path: Option<String>,
    current_project: Option<String>
}

impl Session {
    fn new() -> io::Result<Self> {
        let os = env::consts::OS.to_string();
        let projects_path = match os.as_str() {
            "linux" => {
                let home = env::var("HOME").map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Failed to get HOME: {}", e)))?;
                Some(format!("{}/ramp", home))
            }
            //TODO MACOS here
            "macos" => None, 
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
        let new_path = format!("{}/{}", self.projects_path.as_ref().unwrap_or(&String::new()), name);
        //check that the requested project exists at the specificed path
        if !Path::new(&new_path).exists(){
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to load project, project not found"));
        }
        //check the requested project for compatibility with ramp
        if Path::new(&format!("{}/.ramp", &new_path)).exists() {
            self.current_project = Some(new_path);
            return Ok(());
        }else{
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to load project, not compatible with ramp"));
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
            let mut attempts = 0;
            let max_attempts = 3;
            let mut success = false;
            while attempts < max_attempts && success == false{
                let mut apt_success = false;
                let mut curl_success = false;
                let mut unzip_success = false;
                attempts += 1;
                //update apt
                println!("running sudo apt update... attempt:{}", attempts.to_string());
                let apt_output = Command::new("sudo")
                    .args(["apt", "update"])
                    .output()?;
                println!("apt update stdout: {}", String::from_utf8_lossy(&apt_output.stdout));
                if !apt_output.status.success() {
                    println!("apt update stderr: {}", String::from_utf8_lossy(&apt_output.stderr));
                }else{
                    println!("apt success");
                    apt_success = true;
                }
                //install curl
                println!("installing curl...");
                let curl_output = Command::new("sudo")
                    .args(["apt", "install", "curl", "-y"])
                    .output()?;
                if !curl_output.status.success() {
                    println!("failed to install curl, stderr: {}", String::from_utf8_lossy(&curl_output.stderr));
                }else{
                    println!("curl success");
                    curl_success = true;
                }
                //install unzip
                println!("installing unzip...");
                let unzip_output = Command::new("sudo")
                    .args(["apt", "install", "unzip", "-y"])
                    .output()?;
                if !unzip_output.status.success() {
                    println!("failed to install unzip, stderr: {}", String::from_utf8_lossy(&unzip_output.stderr));
                }else{
                    println!("unzip success");
                    unzip_success = true;
                }
                if unzip_success == true && curl_success == true && apt_success == true{
                    success = true;
                    println!("********apt loop success******")
                }
            }
            if success == false{
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to apt update & install curl and unzip"));
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

    match session.os.as_str() {
        "linux" => {
            // Remove any existing SDK directory (optional, ensures fresh install)
            if Path::new(&sdk_root).exists() {
                println!("Removing existing SDK directory: {}", sdk_root);
                Command::new("rm")
                    .args(&["-rf", &sdk_root])
                    .status()?;
            }

            // Download and install command-line tools
            println!("Installing Android command-line tools...");
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

            // Set up sdkmanager path
            let sdkmanager = format!("{}/latest/bin/sdkmanager", cmdline_tools_dir);

            // Accept licenses
            println!("Accepting Android SDK licenses...");
            let license_output = Command::new(&sdkmanager)
                .args(&["--licenses", "--no-ui"])
                .output()?;
            if !license_output.status.success() {
                println!("License acceptance stderr: {}", String::from_utf8_lossy(&license_output.stderr));
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to accept Android SDK licenses"));
            }

            // Install SDK and NDK packages
            let ndk_package = format!("ndk;{}", desired_ndk_version);
            let packages = vec!["platform-tools", "build-tools;34.0.0", &ndk_package];
            for package in packages {
                println!("Installing {}...", package);
                let install_output = Command::new(&sdkmanager)
                    .args(&[package])
                    .output()?;
                if !install_output.status.success() {
                    println!("Install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
                    return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to install {}", package)));
                }
            }

            // Update PATH
            let current_path = env::var("PATH").unwrap_or_default();
            let new_path = format!(
                "{}:{}/platform-tools:{}/ndk/{}",
                current_path, sdk_root, sdk_root, desired_ndk_version
            );
            env::set_var("PATH", &new_path);
            println!("Android SDK and NDK installed, PATH updated:\n{}", new_path);
        }
        _ => return Err(io::Error::new(io::ErrorKind::Other, "Unsupported OS for Android SDK/NDK installation")),
    }
    Ok(())
}

fn new_project(session: &mut Session, name: &str) -> io::Result<()> {
    //check network connectivity
    println!("Checking for network connectivity...");
    //ping linux servers once to check for connectivity
    let output = Command::new("ping").args(["-c", "1", "linux.org"]).output().unwrap();
    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "No network connection detected"));
    }
    //prepare the template at the target path
    let new_path = format!("{}/{}", session.projects_path.as_ref().unwrap_or(&String::new()), name.to_lowercase());
    match session.os.as_str() {
        "linux" => {
            // Ensure git is installed
            if !is_command_available("git") {
                let mut success = false;
                let mut attempt = 0;
                let max_attempts = 3;
                while success == false && attempt < max_attempts{
                    attempt += 1;
                    println!("git not found. Installing git...");
                    let git_output = Command::new("sudo")
                        .args(&["bash", "-c", "apt install -y git"])
                        .output()?;
                    println!("git install stdout: {}", String::from_utf8_lossy(&git_output.stdout));
                    if !git_output.status.success() {
                        println!("git install stderr: {}", String::from_utf8_lossy(&git_output.stderr));
                    }else{
                        success = true;
                    }
                }
                if success == false{
                    return Err(io::Error::new(io::ErrorKind::Other, "Failed to install git"));
                }
                
            }

            // Create the parent directory if it doesn't exist
            if !Path::new(&new_path).exists() {
                println!("Creating directory: {}", &new_path);
                let mkdir_output = Command::new("mkdir")
                    .args(&["-p", &new_path])
                    .output()?;
                if !mkdir_output.status.success() {
                    println!("mkdir stderr: {}", String::from_utf8_lossy(&mkdir_output.stderr));
                    return Err(io::Error::new(io::ErrorKind::Other, "Failed to create projs directory"));
                }
            }

            // Clone the template repository
            println!("Cloning template from https://github.com/dav-anderson/ramp_template to {}", &new_path);
            let clone_output = Command::new("git")
                .args(&["clone", "https://github.com/dav-anderson/ramp_template", &new_path])
                .output()?;
            println!("git clone stdout: {}", String::from_utf8_lossy(&clone_output.stdout));
            if !clone_output.status.success() {
                println!("git clone stderr: {}", String::from_utf8_lossy(&clone_output.stderr));
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to clone template repository"));
            }

            println!("Template cloned successfully to {}", &new_path);
        }
        _ => return Err(io::Error::new(io::ErrorKind::Other, "Unsupported OS for cloning template")),
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

fn template_naming(session: &mut Session, name: &str) -> io::Result<()> {
    let new_path = format!("{}/{}", session.projects_path.as_ref().unwrap_or(&String::new()), name);
    let capitalized_name = capitalize_first(name);
    //rename dir ios/Webgpu.app
    rename_directory(&format!("{}/ios/Webgpu.app", new_path), &format!("{}.app", &capitalized_name))?;
    //rename ios/Webgpu.app/Info.plist
    let replacements = vec![
        ("Webgpu", capitalized_name.as_str()),
        ("webgpu", name)
    ];
    replace_strings_in_file(&format!("{}/ios/{}.app/Info.plist", new_path, capitalized_name), &replacements)?;
    //rename dir macos/Webgpu.app
    rename_directory(&format!("{}/macos/Webgpu.app", new_path), &format!("{}.app", &capitalized_name))?;
    //rename macos/Webgpu.app/Contents/Info.plist 
    replace_strings_in_file(&format!("{}/ios/{}.app/Info.plist", new_path, capitalized_name), &replacements)?;
    //rename Cargo.toml internals
    let replacements = vec![
        ("webgpu", name),
        ("ramp_template", name)
    ];
    replace_strings_in_file(&format!("{}/ios/{}.app/Info.plist", new_path, capitalized_name), &replacements)?;

    Ok(())
}

fn rename_directory(current_path: &str, target_name: &str) -> io::Result<()> {
    // Get the parent directory of the current path
    let current_dir = Path::new(current_path);
    let parent_dir = current_dir.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Current path has no parent directory")
    })?;

    // Construct the new path by joining the parent directory with the target name
    let new_path = parent_dir.join(target_name);

    // Rename the directory
    fs::rename(current_path, &new_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to rename {} to {}: {}", current_path, new_path.display(), e),
        )
    })?;

    println!("Renamed directory from {} to {}", current_path, new_path.display());
    Ok(())
}

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

fn capitalize_first(s: &str) -> String {
    match s.get(0..1) {
        None => String::new(),
        Some(first) => first.to_uppercase() + &s[1..],
    }
}

fn startup(session: &Session) -> io::Result<()> {
    //check network connectivity
    println!("Checking for network connectivity...");
    //ping linux servers once to check for connectivity
	let output = Command::new("ping").args(["-c", "1", "linux.org"]).output().unwrap();
	if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "No network connection detected"));
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

    //TODO this is broken
    //ENSURE IT checks for existing installation at the path we expect from this installation only (do not search)
    // install_android_sdk_and_ndk(&session)?;

    //TODO install everything for macos

    //Install android SDK & NDK & verify it works (TODO THIS CURRENTLY RUNS EVERYTIME NEED TO FIX)

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

    Ok(())
}

fn main() -> io::Result<()> {
    let mut session = Session::new()?;
    println!("Starting a new session on OS: {}", session.os);
    //TODO commented out for testing only
    // startup(&session);

    //create new proj
    let name: &str = "testproj";
    new_project(&mut session, &name)?;
    println!("current project: {:?}", session.current_project);
    // load_project(&mut session, name)?;
    println!("current project: {:?}", session.current_project);

    //TODOS

    //Single Icon depository with global configuration

    //BUILD for target environments

    //BUILD for simulators

    //finish startup for ubuntu (android) & macos

    //set up key signers for android & ios based on OS

    //WISHLIST

    //ability to use an existing android sdk/ndk installation

    //template version tracking

    //ability to customize projects path

    Ok(())
}


