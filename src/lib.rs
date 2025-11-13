pub mod pages;
pub mod ramp;

use crate::pages::start::StartScreen;

use pelican_ui::start;
use pelican_ui::drawable;
use pelican_ui::layout;
use pelican_ui::drawable::{Drawable, Color, Align};
use pelican_ui::{Application, include_dir, drawables, Component, Context, Plugin, Assets};
use pelican_ui::events::{OnEvent, Event, TickEvent};
use pelican_ui::layouts::{Offset, Stack};
use pelican_ui::components::avatar::{AvatarContent, AvatarIconStyle};
use pelican_ui::components::interface::general::{Bumper, Content, Header, Interface, Page};
use pelican_ui::theme::Theme;
use pelican_ui::components::interface::navigation::{AppPage, RootInfo};
use crate::pages::dashboard::DashboardScreen;
use crate::pages::ios::IOSScreen;
use crate::pages::android::AndroidScreen;
use crate::pages::linux::LinuxScreen;
use crate::pages::macos::MacOSScreen;
use crate::pages::wasm::WASMScreen;
use crate::pages::windows::WindowsScreen;


#[cfg(target_os = "macos")]
#[link(name = "PhotosUI", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "Cocoa", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "AppKit", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "Carbon", kind = "framework")]
extern "C" {}


#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "Metal", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "CoreVideo", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "CoreMedia", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "AVKit", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "AVFoundation", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "Security", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "QuartzCore", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "c++")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "AudioToolbox", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "Foundation", kind = "framework")]
extern "C" {}

// Define the main application struct entry point.
pub struct RampGUI;

// Implement the Application trait for RampGUI
impl Application for RampGUI {
    // Asynchronously create the main drawable UI component
    fn interface(ctx: &mut Context) -> Interface {
        // Create the navigation bar
        let home = RootInfo::icon("home", "home", StartScreen::new(ctx).ok().unwrap());
        let dashboard = RootInfo::icon("car", "dashboard", DashboardScreen::new(ctx).ok().unwrap());
        let ios = RootInfo::icon("phone", "ios", IOSScreen::new(ctx).ok().unwrap());
        let android = RootInfo::icon("phone", "android", AndroidScreen::new(ctx).ok().unwrap());
        let macos = RootInfo::icon("phone", "macos", MacOSScreen::new(ctx).ok().unwrap());
        let windows = RootInfo::icon("phone", "windows", WindowsScreen::new(ctx).ok().unwrap());
        let linux = RootInfo::icon("phone", "linux", LinuxScreen::new(ctx).ok().unwrap());
        let wasm = RootInfo::icon("phone", "wasm", WASMScreen::new(ctx).ok().unwrap());
        
        // Create the main interface with navgiation bar
        Interface::new(ctx, vec![home, dashboard, ios, android, macos, windows, linux, wasm])
    }

    //provide a global theme
    fn theme(assets: &mut Assets) -> Theme {
        assets.include_assets(include_dir!("./assets/resources"));
        Theme::dark(assets, Color::from_hex("#ff1f23", 255))
    }
}

// Macro to start the application
start!(RampGUI);