use super::helper::{is_command_available, capitalize_first};
use super::{Session};
use image::{self, imageops, DynamicImage, ImageEncoder};
use std::env;
use std::fs;
use std::fs::{File, read_to_string};
use std::io::{self, Write, BufReader, BufRead, ErrorKind};
use std::path::Path;
use std::process::{Command, Stdio};
use regex::Regex;
use std::thread::sleep;
use std::time::{Duration, Instant};

//sign an app build
fn sign_build(session: &mut Session, target_os: &str, release: bool) -> io::Result<()> {
    println!("signing app bundle for {}", target_os);
    if target_os == "ios" {
        //check if keychain is locked, if so, unlock
        loop{
            println!("looping keychain check");
            let output = Command::new("security")
                .args(["show-keychain-info", &format!("{}/login.keychain-db", session.get_path("keystore_path")?)])
                .output()
                .unwrap();
            if String::from_utf8_lossy(&output.stdout).contains("could not be found"){
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("could not find the requested keychain"),
                ));
            }
            if !String::from_utf8_lossy(&output.stdout).contains("locked") {
                break;
            }else{
                // wait 3 seconds
                sleep(Duration::from_secs(3));
            }
        }
        //sign the build
        let app_bundle = format!("{}/{}/ios/{}.app", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), capitalize_first(session.current_project.as_ref().unwrap()));
        let output = Command::new("codesign")
        .args(["--force", "--sign", session.certs.macos.as_ref(), "--entitlements", &format!("{}/entitlements.plist", &app_bundle),  &app_bundle])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
        if !output.status.success(){
            let error = String::from_utf8_lossy(&output.stderr);
            if !error.contains("failed to parse entitlements"){
                println!("App bundle not signed, missing provisioning entitlements. Continuing to provisioning");
                return Ok(())
            }
            else{
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("ios post build failed to sign app bundle: {}", error),
                ));
            }
        }
    }
    //TODO add support for other outputs
    println!("signed {} app bundle", target_os);
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

            session.update_current_project(name.to_string())?;

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
    let default_bundle = format!("com.ramp.{}", name);
    let replacements = vec![("Ramp", capitalized_name.as_str()), ("ramp", name), ("com.example.name", default_bundle.as_str())];
    //rename default strings in cargo.toml
    replace_strings_in_file(&format!("{}/Cargo.toml", new_path), &replacements)?;
    //rename dir ios/Ramp.app
    rename_directory(
        &format!("{}/ios/Ramp.app", new_path),
        &format!("{}.app", &capitalized_name),
    )?;
    //rename default strings in ios/Ramp.app/Info.plist
    replace_strings_in_file(
        &format!("{}/ios/{}.app/Info.plist", new_path, capitalized_name),
        &replacements,
    )?;
    //rename dir macos/Ramp.app
    rename_directory(
        &format!("{}/macos/Ramp.app", new_path),
        &format!("{}.app", &capitalized_name),
    )?;
    //rename default strings in macos/Ramp.app/Contents/Info.plist
    replace_strings_in_file(
        &format!("{}/macos/{}.app/Contents/Info.plist", new_path, capitalized_name),
        &replacements,
    )?;
    //replace bundle id if applicable
    if bundle_id.is_some() {
        let existing_bundle = format!("com.ramp.{}", name);
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

fn get_bundle_id(session: &mut Session, target_os: &str) -> io::Result<String> {
    let plist_path = format!("{}/{}/{}/{}.app/Info.plist", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), &target_os, capitalize_first(session.current_project.as_ref().unwrap()));
    let plist_content = fs::read_to_string(&plist_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to read Info.plist in get_bundle_id: {}", e)))?;

    let re = Regex::new(r#"<key>CFBundleIdentifier</key>\s*<string>([^<]+)</string>"#).unwrap();
    let captures = re.captures(&plist_content)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "CFBundleIdentifier not found"))?;

    let bundle_id = captures.get(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Failed to extract CFBundleIdentifier"))?
        .as_str()
        .to_string();

    Ok(bundle_id)
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

fn provision_device(session: &mut Session, udid: String, target_os: &String, release: bool) -> io::Result<()> {
    println!("Provisioning a new device with unique device id: {}", &udid);
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
    println!("{}", &udid);
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
    println!("Provisioning profile for device id: {} and app bundle: {}", &udid, &bundle_id);
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

     println!("7. Select the device profile corresponding to UUID: {}", &udid);
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
    //install the profile to the device with the UDID and ilibmobiledevice
    println!("installing provisioning profile to the target device");
    let ilibimobile_bin = format!("{}/ideviceprovision", session.get_path("homebrew_path")?);
    println!("ideviceprovision path: {}", &ilibimobile_bin);
    let mobile_provision_path = format!("{}/{}", &mp_destination, &mobileprovision_file);
    println!("Mobile provision path: {}", &mobile_provision_path);
    let output = Command::new(&ilibimobile_bin)
        .args(["install", &mobile_provision_path, "--udid", &udid])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to install .mobileprovision to the device"));
    }

    //decode and extract entitlements from the mobile provision into an entitltements.plist
    println!("decoding and extracting the entitlements from the mobile provision");
    let security_output = Command::new("security")
    .args(["cms", "-D", "-i", &mobile_provision_path])
    .output()?;
    if !security_output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to decode provisioning profile"));
    }
    let entitlements_path = format!("{}/entitlements.plist", &mp_destination);

    let mut plutil_cmd = Command::new("plutil")
        .args(["-extract", "Entitlements", "xml1", "-o", &entitlements_path, "-"])
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = plutil_cmd.stdin.take() {
        stdin.write_all(&security_output.stdout)?;
    }

    let plutil_output = plutil_cmd.wait_with_output()?;

    if !plutil_output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to extract entitlements"));
    }
    println!("Successfully decoded and extracted entitlements to entitlements.plist");
    println!("Signing app bundle with new provisioning");
    //resign the build with new provisioning parameters outlined (this is normally done within the build output flow)
    sign_build(session, &target_os, release)?;
    Ok(())
}

fn get_udid_by_target(device_target: &str) -> io::Result<String> {
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

//deprecated
// fn load_simulator(session: &Session, target_os: String) -> io::Result<()>{
//     println!("load_simulator");
//     if target_os == "ios" {
//         //TODO make sure this never tried to boot a non sim binary
//         println!("deploying to {} simulator", target_os);
//         //TODO check if simulator is already running first
//         //open ios simuator
//         let output = Command::new("open")
//             .args(["-a", "simulator"])
//             .output()
//             .unwrap();
//         if !output.status.success(){
//             return Err(io::Error::new(
//                 io::ErrorKind::Other,
//                 "could not open IOS simulator: {}",
//             ));
//         }
//         //TODO create a device, need to build out support here
//         //boot & install the .app bundle to the simulator
//         let output = Command::new("xcrun")
//             .args(["simctl", "install", "booted", &format!("{}/{}/ios/{}.app", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), capitalize_first(session.current_project.as_ref().unwrap()))])
//             .output()
//             .unwrap();
        
//         if !output.status.success(){
//             return Err(io::Error::new(
//                 io::ErrorKind::Other,
//                 "could not deploy to IOS simulator: {}",
//             ));
//         }
//     }
//     //TODO MACOS side
//     //macos sim
//     //ios sim
//     //android sim
//     //windows sim
//     //ubuntu sim?
//     //wasm?

//     //TODO UBUNTU SIDE
//     //android sim
//     //windows sim
//     //ubuntu sim?
//     //wasm?
//     println!("finished deploying to {} simulator", target_os);
//     Ok(())
// }

fn is_device_provisioned(session: &mut Session, app_bundle_path: &str, device_id: &str, udid: &str) -> io::Result<bool> {
    println!("checking if target device is properly provisioned");
    //obtain the mobile provision file name
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
                println!("No provisioning profile found");
                return Ok(false);
            }
            1 => {
                println!("Exactly one provisioning profile found");
                mobileprovision_file = matching_files[0].clone();
            }
            _ => {
                println!("something weird happened finding the provisioning profile");
                return Ok(false);
            }
        }
    // 
    
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
    let device_entry = format!("<string>{}</string>", udid);
    //check if the profile contains the device id
    println!("checking if array content: {:?} contains device entry: {:?}", &array_content, &device_entry);
    if array_content.contains(&device_entry) {
        println!("provisioning profile contains the device id...checking device for installation");
        //check that the profile is installed on the device
        if let Some(key_pos) = xml.find("<key>Name</key>") {
            if let Some(string_start) = xml[key_pos..].find("<string>") {
                let start = key_pos + string_start + "<string>".len();
                if let Some(string_end) = xml[start..].find("</string>") {
                    let profile_name = xml[start..start + string_end].trim().to_string();
                    if !profile_name.is_empty() {
                        //list the installed provisions
                        let output = Command::new(format!("{}/ideviceprovision", session.get_path("homebrew_path")?))
                            .args(["list", "--udid", udid])
                            .output()?;
                        if !output.status.success() {
                            return Err(io::Error::new(io::ErrorKind::Other, "failed to list provisioning profiles: {}",));
                        }
                        let profiles = String::from_utf8_lossy(&output.stdout);
                        if profiles.contains(&profile_name) {
                            println!("target device is already provisioned");
                            return Ok(true)
                        }else{
                            println!("provisioning profile is not currently installed on the target device");
                            return Ok(false)
                        }
                    }
                }
            }
        }
        return Err(io::Error::new(io::ErrorKind::Other, "Name not found in provisioning profile: {}",));
    } else {
        println!("target device is not provisioned");
        return Ok(false)
    }let profile_path_str = format!("{}/{}", &app_bundle_path, &mobileprovision_file);
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
    let device_entry = format!("<string>{}</string>", udid);
    //check if the profile contains the device id
    println!("checking if array content: {:?} contains device entry: {:?}", &array_content, &device_entry);
    if array_content.contains(&device_entry) {
        println!("provisioning profile contains the device id...checking device for installation");
        //check that the profile is installed on the device
        if let Some(key_pos) = xml.find("<key>Name</key>") {
            if let Some(string_start) = xml[key_pos..].find("<string>") {
                let start = key_pos + string_start + "<string>".len();
                if let Some(string_end) = xml[start..].find("</string>") {
                    let profile_name = xml[start..start + string_end].trim().to_string();
                    if !profile_name.is_empty() {
                        //list the installed provisions
                        let output = Command::new(format!("{}/ideviceprovision", session.get_path("homebrew_path")?))
                            .args(["list", "--udid", udid])
                            .output()?;
                        if !output.status.success() {
                            return Err(io::Error::new(io::ErrorKind::Other, "failed to list provisioning profiles: {}",));
                        }
                        let profiles = String::from_utf8_lossy(&output.stdout);
                        if profiles.contains(&profile_name) {
                            println!("target device is already provisioned");
                            return Ok(true)
                        }else{
                            println!("provisioning profile is not currently installed on the target device");
                            return Ok(false)
                        }
                    }
                }
            }
        }
        return Err(io::Error::new(io::ErrorKind::Other, "Name not found in provisioning profile: {}",));
    } else {
        println!("target device is not provisioned");
        Ok(false)
    }
}

fn deploy_usb_tether(session: &mut Session, target_os: String) -> io::Result<()> {
    //deploy to target device
    if target_os == "ios"{
        //obtain device uuid
        let udid = get_udid_by_target("iphone")?;
        let device_id = get_device_identifier()?;
        println!("target device UDID: {}", &udid);
        println!("deploying to ios device ID: {}", &device_id);
        //check for an existing provisioning profile
        let profile_path_str = &format!("{}/{}/{}/{}.app", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), &target_os, capitalize_first(session.current_project.as_ref().unwrap()));
        let device_provisioned = is_device_provisioned(session, &profile_path_str, &device_id, &udid)?;
        if !device_provisioned {
            //add a new provisioning profile for a macos device
            provision_device(session, udid, &target_os, false)?;
        }
        let output = Command::new("xcrun")
            .args(["devicectl", "device", "install", "app", "--device", &device_id, &format!("{}/{}/ios/{}.app", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), capitalize_first(session.current_project.as_ref().unwrap()))])
            .output()
            .unwrap();
        if !output.status.success() {
            println!("here is the output: {:?}", &output);
            return Err(io::Error::new(io::ErrorKind::Other, "could not install app bundle to IOS device via USB tether: {}"));
        }
        let bundle_id = get_bundle_id(session, "ios")?;
        println!("Deploying bundle id: {} to device: {}", &bundle_id, &device_id);
        let output = Command::new("xcrun")
            .args(["devicectl", "device", "process", "launch", "--device", &device_id, &bundle_id])
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "could not launch app bundle to IOS device via USB tether"));
        }
    }else if target_os == "android"{
        //android device tether deployment
        let adb_path = format!("{}/adb", session.get_path("platform_tools_path")?);
        if !is_android_device_connected(&adb_path){
            return Err(io::Error::new(io::ErrorKind::Other, "no android device detected, or multiple devices connected"));
        }
        println!("one android device detected");
        //obtain the apk_name & package value from the Cargo.toml
        let cargo_toml = read_to_string(format!("{}/{}/Cargo.toml", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap()))?;
        let mut apk_name = String::new();
        let mut package_name = String::new();
        for line in cargo_toml.lines() {
        let trim = line.trim();
            if trim.starts_with("apk_name =") {
                if let Some(start) = trim.find('=') {
                    let value = &trim[start + 1..].trim();
                    apk_name = value.trim_matches(|c| c == '"' || c == '\'').to_string();
                }
            }else if trim.starts_with("package =") {
                if let Some(start) = trim.find('=') {
                    let value = &trim[start + 1..].trim();
                    package_name = value.trim_matches(|c| c == '"' || c == '\'').to_string();
                }
            }
        }
        if apk_name.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "apk_name value not found in cargo.toml"));
        }
        if package_name.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "package value not found in cargo.toml"));
        }
        println!("The APK name is: {}", apk_name);
        println!("The package name is: {}", package_name);
        //path to the apk
        let apk_path = format!("{}/{}/target/debug/apk/{}.apk", session.projects_path.as_ref().unwrap(), session.current_project.as_ref().unwrap(), &apk_name);
        println!("apk path: {}", apk_path);
        //install the apk
        let output = Command::new(&adb_path)
            .args(["install", "-r", &apk_path])
            .output()
            .unwrap();
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "could not install the apk"));
        }
        println!("APK installed!");
        //TODO obtain the abd_payload which needs the activity name?
        // let adb_payload = get_adb_launch_payload(session, Path::new(&apk_path))?;
        // println!("adb payload: {}", adb_payload);

        //TODO launch the adb payload after valid install


        //TODO need to determine the app bundle name and extension programmatically for auto launch
        //we can launc the app with `adb shell am start -n "com.your.bundle.id/.MainActivity"`
        //we can uninstall with `adb uninstall com.bundle.id`
    }

    println!("Successfully deployed to {} device", &target_os);

    Ok(())
    
}

//TODO "main activity not found", might remove this LLM code
fn get_adb_launch_payload(session: &mut Session, apk_path: &Path) -> Result<String, io::Error> {
    // Run aapt to dump manifest as text tree
    let aapt_path = format!("{}/aapt", session.get_path("build_tools_path")?);
    let output = Command::new(&aapt_path)
        .args(["dump", "xmltree", apk_path.to_str().unwrap(), "AndroidManifest.xml"])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "aapt command failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let reader = BufReader::new(stdout.as_bytes());
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    let mut package = None;
    let mut main_activity = None;

    for line in lines.iter() {
        // Extract package from root <manifest> attribute
        if line.trim().starts_with("A: package=") {
            if let Some(start) = line.find('\"') {
                if let Some(end) = line[start + 1..].find('\"') {
                    package = Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }

        // Find main activity: Look for <intent-filter> with MAIN and LAUNCHER under <activity>
        if main_activity.is_none() && line.trim().starts_with("E: activity") {
            let mut in_activity = true;
            let mut activity_name = None;
            let mut has_main = false;
            let mut has_launcher = false;

            // Parse sub-lines for this activity block
            let activity_idx = lines.iter().position(|l| l == line).unwrap_or(0);
            for sub_line in lines.iter().skip(activity_idx + 1) {
                let trim_sub = sub_line.trim();
                if trim_sub.starts_with("A: android:name") {
                    if let Some(start) = sub_line.find('\"') {
                        if let Some(end) = sub_line[start + 1..].find('\"') {
                            activity_name = Some(sub_line[start + 1..start + 1 + end].to_string());
                        }
                    }
                } else if trim_sub.starts_with("E: intent-filter") {
                    // Check for MAIN action and LAUNCHER category in this filter
                    let filter_idx = lines.iter().position(|l| l == sub_line).unwrap_or(0);
                    for filter_line in lines.iter().skip(filter_idx + 1) {
                        let trim_filter = filter_line.trim();
                        if trim_filter.starts_with("E: ") && !trim_filter.starts_with("E: action") && !trim_filter.starts_with("E: category") {
                            break; // End of this intent-filter
                        }
                        if trim_filter.contains("android.intent.action.MAIN") {
                            has_main = true;
                        }
                        if trim_filter.contains("android.intent.category.LAUNCHER") {
                            has_launcher = true;
                        }
                    }
                } else if trim_sub.starts_with("E: ") {
                    in_activity = false; // End of activity block
                    break;
                }
            }

            if has_main && has_launcher {
                if let Some(name) = activity_name {
                    let full_name = if name.starts_with('.') {
                        format!("{}{}", package.as_ref().unwrap_or(&String::new()), name)
                    } else {
                        name
                    };
                    main_activity = Some(full_name);
                }
            }
        }

        if package.is_some() && main_activity.is_some() {
            break;
        }
    }

    let pkg = package.ok_or(io::Error::new(io::ErrorKind::NotFound, "Package not found"))?;
    let act = main_activity.ok_or(io::Error::new(io::ErrorKind::NotFound, "Main activity not found"))?;

    Ok(format!("-n \"{}/{}\"", pkg, act))
}

fn is_android_device_connected(adb_path: &str) -> bool {
    println!("adb path: {}", adb_path);
    let output = match Command::new(adb_path)
        .arg("devices")
        .output() {
        Ok(out) => out,
        Err(_) => return false,
    };

    if !output.status.success() {
        println!("failed to run adb devices");
        return false;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // Skip the header line "List of devices attached"
    let device_lines: Vec<&str> = lines.into_iter()
        .skip(1) // Skip header
        .filter(|line| line.trim().ends_with("device")) // Only count authorized "device" status
        .collect();

    device_lines.len() == 1
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
    //map the build target to an output path
    let mut output_path = String::new();
    //building for debug
    if !release{
        output_path = match target_os.as_str() {
            "windows" => format!(
                "{}/target/x86_64-pc-windows-gnu/debug/{}.exe", &project_path, session.current_project.as_ref().unwrap()
            ),
            //TODO fix running on linux
            "linux" => if session.os.as_str() == "linux" {format!(
                    "{}/target/debug/TODO NEED TO FIX THIS", &project_path, 
                    //running on mac
                )} else {format!(
                    "{}/target/aarch64-unknown-linux-gnu/debug/{}", &project_path, session.current_project.as_ref().unwrap()
                )},
            "wasm" => format!(
                "{}/target/wasm32-unknown-unknown/debug/main.wasm", &project_path
            ),
            "android" => format!(
                "{}/target/debug/apk/{}.apk", &project_path, capitalize_first(session.current_project.as_ref().unwrap())
            ),
            "android_run" => format!(
                "{}/target/debug/apk/{}.apk", &project_path, capitalize_first(session.current_project.as_ref().unwrap())
            ),
            "ios" => if session.os.as_str() == "macos" {format!(
                "{}/target/aarch64-apple-ios/debug/{} ...if you are looking for the full app bundle check the ramp/{}/ios directory", &project_path, session.current_project.as_ref().unwrap(), session.current_project.as_ref().unwrap()
            )} else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported target OS: {}", target_os),
                ))
            },
            "macos" => if session.os.as_str() == "macos" {format!(
                "{}/target/debug/{}", &project_path, session.current_project.as_ref().unwrap()
                )}  else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unsupported target OS: {}", target_os),
                    ))
                },
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported target OS: {}", target_os),
                ))
            }
        };
    //TODO need to update when building for release
    //building for release
    } else {
        output_path = format!("TODO output paths for release builds");
    }
   

    // Map the build target to a Cargo command payload
    let cargo_args = match target_os.as_str() {
        "windows" => format!(
            "build --target x86_64-pc-windows-gnu{}",
            if release { " --release " } else { "" }
        ),
        "linux" => if session.os.as_str() == "linux" {format!(
            "build{}", if release { " --release " } else { "" }
            //TODO need to fix this when running on macos and building for linux
            )} else {format!(
                "build --target aarch64-unknown-linux-gnu{}", if release { " --release " } else { "" }
            )},
        "wasm" => format!(
            "build --lib --target wasm32-unknown-unknown{}",
            if release { " --release " } else { "" }
        ),
        "android" => format!(
            "apk build{}",
            if release { " --release " } else { " --lib " }
        ),
        "android_run" => format!(
            "apk run{}",
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
    let cargo_command = format!("{} {}", session.get_path("cargo_path")?, cargo_args);
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", session.get_path("homebrew_path")?, current_path);
    println!("building for {} on {}", &target_os, session.os);
    let output = if target_os.as_str() == "android" || target_os.as_str() == "android_run"{
                    //building for android
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
                //building for linux on macos
                } else if session.os == "macos" && target_os.as_str() == "linux" {
                    Command::new("bash")
                    .arg("-c")
                    //insert path to zigbuild
                    .arg(format!("{} {}", session.get_path("zigbuild_path")?, &cargo_args))
                    .current_dir(project_dir) // Set working directory
                    //provide the temp environment path for zig
                    .env("PATH", new_path)                
                    .stdout(Stdio::inherit()) // Show build output
                    .stderr(Stdio::inherit())
                    .output()?
                //all other build cases
                } else {
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
    println!("The binary can be found at {}", &output_path);

    //post build house keeping
    //debug post build
    if !release{
        if target_os == "ios"{
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
            sign_build(session, &target_os, release)?;
            println!("post build complete; resigned ios app bundle: {:?}", output);
        }else if target_os == "android" {
            println!("TODO android debug postbuild");
        }else if target_os == "windows" {
            println!("TODO windows debug postbuild");
            //TODO windows post build
            //icons & app bundle
        }else if target_os == "macos" {
            println!("TODO macos debug postbuild");
            //TODO macos post build
            //combine chipset architecture
            //icons and app bundle 
        }else if target_os == "wasm"{
            println!("TODO wasm debug postbuild");
        }else if target_os == "linux"{
            println!("TODO linux debug postbuild");
        }
    //release post build
    } else {
        println!("TODO post build for release");
    }
    
    //deprecated
    // else if target_os == "ios_sim" {
    //     println!("performing ios sim post build...");
    //     let output = Command::new("cp")
    //     .args([&format!("{}/target/aarch64-apple-ios-sim/debug/{}", project_path, session.current_project.as_ref().unwrap()), &format!("{}/ios/{}.app/", project_path, capitalize_first(session.current_project.as_ref().unwrap()))])
    //     .output()
    //     .unwrap();
    //     if !output.status.success() {
    //         let error = String::from_utf8_lossy(&output.stderr);
    //         return Err(io::Error::new(
    //             io::ErrorKind::Other,
    //             format!("ios_sim post build failed: {}", error),
    //         ));
    //     }
    // } 

    //TODO compile windows app.rc for desktop icon, see ramp_template readme
    Ok(())
}

fn get_ios_sdk() -> io::Result<String> {
    let output = Command::new("xcrun")
    .args(["--sdk", "iphoneos", "--show-sdk-path"])
    .output()
    .map_err(|e| {
        io::Error::new(
            ErrorKind::Other,
            format!("Failed to execute xcrun to obtain SDKROOT: {}", e),
        )
    })?;

    if !output.status.success() {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("xcrun xctrace failed: {}", String::from_utf8_lossy(&output.stderr)),
        ));
    }

    let sdk_path = String::from_utf8_lossy(&output.stdout);
    println!("SDK PATH FOUND: {}", &sdk_path);
    Ok(sdk_path.trim().to_string())
}

//old main()
 // let mut session = Session::new()?;
    // println!("Starting a new session on OS: {}", session.os);
    // session.get_all_paths()?;

    // // Collect all command-line arguments into a Vec<String>
    // let args: Vec<String> = env::args().collect();

    // // Print arguments for debugging
    // println!("Arguments: {:?}", args);
    
    // // Check for the -install argument, this flow requires sudo priveleges
    // if args.contains(&"-install".to_string()) {
    //     println!("Running install with elevated privileges...");
    //     //initial install
    //     install(&mut session)?;
    //     //TODO move the binary from the .dmg or the .deb after install is finished
    //     //TODO terminate the session
    //     //TODO can we start an external script with a timer here to relaunch ramp gui after closing initial install client?
    // }else{
    //     //create new proj
    //     let name: &str = "testproj";
    //     // new_project(&mut session, &name)?;
    //     println!("current project: {:?}", session.current_project);

    //     //load an existing proj
    //     load_project(&mut session, name)?;
    //     println!("current project: {:?}", session.current_project);

    //     // //format the icon.png in assets/resources/icons across all outputs
    //     update_icons(&session)?;

    //     // //build the target output (session: &Session, target_os: String, release: bool)
    //     build_output(&mut session, "macos".to_string(), false)?;

    //     //TODO can build_output(&mut session, "android_run".to_string(), false)?; replace the entire deploy_usb_tether flow for android? It seems that way.


    //     // // load_simulator(&mut session, "ios".to_string())?;
    //     // deploy_usb_tether(&mut session, "ios".to_string())?;
    // }



     //TODOS

    //update the ramp_template with missing file architecture
    //programmatically introduce xcode frameworks as required by pelican ui on new template instances

    //test that all app icons are properly removed and recreated after an update    

    //MACOS
    //fix windows post build 
    //fix ubuntu output compatability (see notes in install function)
    //linux post build?
    //lipo outputs for combined chipset architectures for ios simulator and macos release
    //set up/config key signers & dev certs for releases


    //LINUX
    //start to finish comb through
    //ensure that all commands set paths in the .ramp config
    //rework all commands to use paths from the .ramp config
    //discard any .zsh or .bshrc persistence
    //refactor all sudo requirements outside of the -install flag, consider a .deb install script that calls sudo with an -install flag
    //setup/config key signers
    //hot load android over a usb


    //WISHLIST

    //signing keys and dev cert management tools

    //simulators for every build output

    //gracefully intercept and handle errors where the user's OS is out of date (particularly in the case of MacOS)

    //ability to use an existing android sdk/ndk installation

    //more robust version specification for critical components (xcode, ios ndk, jdk, android ndk & sdk, etc etc)

    //template version tracking

    //ability to customize projects path

    //ability to customize paths to binaries (can manually do this in "$HOME/.ramp" config currently)


