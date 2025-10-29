use super::helper::{is_command_available, is_xcode_tools_installed, get_user_home};
use super::session::{Session};
use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

//initialization function upon starting the app
//WARNING install must only be run with sudo privleges
pub fn install() -> io::Result<()> {
    let mut session = Session::new()?;
    println!("Starting a new session on OS: {}", session.os);
    //populate any pre-existing config paths
    session.get_all_paths()?;
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
        install_rustup(&mut session)?;
    } else {
        println!("rustup is installed.");
    }

    // Check if cargo is installed
    if !is_command_available(&session.get_path("cargo_path")?) {
        println!("cargo not found. Running rustup to ensure full toolchain...");
        install_rust_toolchain(&mut session)?;
    } else {
        println!("cargo is installed.");
    }

    println!("Rust toolchain is ready!");

    //Install OS appropriate build targets
    install_build_targets(&mut session)?;

    //install mac/ios toolchains
    install_macos_ios_toolchains(&mut session)?;

    //install android toolchains
    install_android_toolchains(&mut session)?;

    //setup keychain
    setup_keychain(&mut session)?;

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

// Function to ensure full Rust toolchain is installed via rustup
pub fn install_rust_toolchain(session: &mut Session) -> io::Result<()> {
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

//function to create the .ramp config file
pub fn create_ramp_config(session: &Session) -> io::Result<()> {
    let config_path = format!("{}/.ramp", session.home);

    //create the file if it doesn't exist
    if !Path::new(&config_path).exists(){
        File::create(&config_path)?;
    }

    Ok(())
}

// Function to install rustup
pub fn install_rustup(session: &mut Session) -> io::Result<()> {
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

//install build targets for all supported ramp outputs
pub fn install_build_targets(session: &mut Session) -> io::Result<()> {
    println!("Detected OS: {}", session.os);

    let mac_targets: Vec<String> = vec![
        "aarch64-apple-ios".to_string(),
        "x86_64-apple-ios".to_string(),
        "aarch64-apple-ios-sim".to_string(),
        "x86_64-apple-darwin".to_string(),
        "aarch64-apple-darwin".to_string(),
        "x86_64-pc-windows-gnu".to_string(),
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

//install homebrew on macos
pub fn install_homebrew(session: &mut Session) -> io::Result<()> {
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

//prompt the user to first install the xcode app on macos via the apple app store
pub fn install_xcode_prompt() -> io::Result<()> {
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

//installs the toolchains for macos and ios development
pub fn install_macos_ios_toolchains(session: &mut Session) -> io::Result<()> {
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

    //brew install libimobiledevice
    let output = Command::new(format!("{}/brew", session.get_path("homebrew_path")?))
        .args(["install", "libimobiledevice"])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to install libimobiledevice"));
    }

    //install mingw-w64
    let output = Command::new(format!("{}/brew", session.get_path("homebrew_path")?))
    .args(["install", "mingw-w64"])
    .output()?;
    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to install mingw-w64 windows linker"));
    }

    // Install zigbuild
    println!("Installing cargo-zigbuild...");
    let install_output = Command::new(session.get_path("cargo_path")?)
        .args(&["install", "--locked", "cargo-zigbuild"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;
    if !install_output.status.success() {
        println!("cargo install stderr: {}", String::from_utf8_lossy(&install_output.stderr));
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to install cargo-zigbuild"));
    }
    println!("cargo-zigbuild installed successfully.");

    //set the zigbuild path
    session.set_path("zigbuild_path", format!("{}/.cargo/bin/cargo-zigbuild", session.home))?;

    //brew install zig
    let output = Command::new(format!("{}/brew", session.get_path("homebrew_path")?))
    .args(["install", "zig"])
    .output()?;
    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to install zig"));
    }

    //add mingw-w64 linker to the global .cargo config
    let config_path = format!("{}/.cargo/config.toml", get_user_home()?);
    let mut file = File::create(config_path)?;
    let config_payload = format!("[target.x86_64-pc-windows-gnu]\nlinker = \"{}/x86_64-w64-mingw32-gcc\"\n", session.get_path("homebrew_path")?);
    file.write_all(config_payload.as_bytes())?;

    println!("MacOS & IOS toolchain installation complete");
    Ok(())
}

//deprecated can likely be removed
// fn install_simulators(session: &Session) -> io::Result<()>{
//     if session.os.as_str() == "macos"{
//         //run xcrun simctl list devices to initialize
//         println!("setting up simulators");
//         println!("setting up xcrun simctl");
//         let output = Command::new("sudo")
//         .args(["xcrun", "simctl", "list", "devices"])
//         .output()
//         .unwrap();
//         println!("result: {:?}", output);
//         if !output.status.success() {
//             return Err(io::Error::new(io::ErrorKind::Other, "Failed to initialize xcode simulator"));
//         }
//     }
    
//     Ok(())
// }

//configure a keychain profile for signing apps for distribution
pub fn setup_keychain(session: &mut Session) -> io::Result<()>{
    //TODO this currently creates a debug signing certificate only, different certificate properties must be configured for release on apple's developer website
    println!("keychain installer");
    if session.os.as_str() == "macos"{
        //check if keychain is locked, if so, unlock
        loop{
            let output = Command::new("security")
                .args(["show-keychain-info", &format!("{}/login.keychain-db", session.get_path("keystore_path")?)])
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
        let key_path = format!("{}/ramp.pem", session.get_path("keystore_path")?);
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
            .args(["import", &format!("{}/ramp.pem", session.get_path("keystore_path")?), "-k", &format!("{}/login.keychain-db", session.get_path("keystore_path")?), "-T", "/usr/bin/codesign"])
            .output()
            .unwrap();
            if !output.status.success() {
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to import the ramp.pem to the keychain-db"));
            }
        }else {println!("ramp.pem already exists");}
        //check if the CSR exists, if not generate a new CSR
        let csr_path = format!("{}/ramp.csr", session.get_path("keystore_path")?);
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
            .arg(session.get_path("keystore_path")?)
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
                    .args([&cert_download, &session.get_path("keystore_path")?])
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
        //security import the cert into the keychain
        let output = Command::new("security")
            .args(["import", &format!("{}/development.cer", session.get_path("keystore_path")?), "-k", &format!("{}/login.keychain-db", session.get_path("keystore_path")?)])
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to import the development.pem to the keychain-db"));
        }   
        //get the App Developer Worldwide Developer Relations Ceritifcation Authority certificate
        let output = Command::new("curl")
            .args(["-o", &format!("{}/AppleWWDRCA.cer", session.get_path("keystore_path")?), "https://www.apple.com/certificateauthority/AppleWWDRCAG3.cer"])
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to obtain Apple WWDRCA cert with curl"));
        }  
        //add the apple Developer worldwide relations cert to the security chain
        let output = Command::new("security")
        .args(["import", &format!("{}/AppleWWDRCA.cer", session.get_path("keystore_path")?), "-k", &format!("{}/login.keychain-db", session.get_path("keystore_path")?)])
        .output()
        .unwrap();
        println!("AppleWWDRCA.cer import output: {:?}", output);
        if !output.status.success() && !String::from_utf8_lossy(&output.stderr).contains("already exists") {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to import the AppleWWDRCA.cer to the keychain-db"));
        }
        //get the App Developer Worldwide Developer Relations Ceritifcation Authority certificate
        let output = Command::new("curl")
        .args(["-o", &format!("{}/AppleRootCA.cer", session.get_path("keystore_path")?), "https://www.apple.com/certificateauthority/AppleRootCA-G3.cer"])
        .output()
        .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to obtain AppleRootCA cert with curl"));
        }  
        //add the apple Root CA cert to the security chain
        let output = Command::new("security")
        .args(["import", &format!("{}/AppleRootCA.cer", session.get_path("keystore_path")?), "-k", &format!("{}/login.keychain-db", session.get_path("keystore_path")?)])
        .output()
        .unwrap();
        println!("AppleRootCA.cer import output: {:?}", output);
        if !output.status.success() && !String::from_utf8_lossy(&output.stderr).contains("already exists") {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to import the AppleRootCA.cer to the keychain-db"));
        }
        //Get the Developer ID CA
        let output = Command::new("curl")
        .args(["-o", &format!("{}/AppleDevIDCA.cer", session.get_path("keystore_path")?), "https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer"])
        .output()
        .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to obtain AppleDevIDCA.cer cert with curl"));
        }  
        //add the apple Root CA cert to the security chain
        let output = Command::new("security")
        .args(["import", &format!("{}/AppleDevIDCA.cer", session.get_path("keystore_path")?), "-k", &format!("{}/login.keychain-db", session.get_path("keystore_path")?)])
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

//install toolchains for android development
pub fn install_android_toolchains(session: &mut Session) -> io::Result<()> {
    println!("Setting up Android SDK and NDK for {}", session.os);
    session.set_path("sdk_path", format!("{}/Android/sdk", session.home))?;
    session.set_path("cmdline_tools_path", format!("{}/Android/sdk/cmdline-tools", session.home))?;
    session.set_path("build_tools_path", format!("{}/Android/sdk/build-tools/34.0.0", session.home))?;
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
            .args(&["install", "--locked", "cargo-apk"])
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