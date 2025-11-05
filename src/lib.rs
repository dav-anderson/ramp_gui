pub mod pages;
pub mod ramp;

use crate::pages::start::StartScreen;

use pelican_ui::start;
use pelican_ui::drawable;
use pelican_ui::layout;
use pelican_ui::drawable::{Drawable, Color, Align};
use pelican_ui::{include_dir, drawables, Component, Context, Application, Plugin};
use pelican_ui::events::{OnEvent, Event, TickEvent};
use pelican_ui::layouts::{Offset, Stack};
use pelican_ui::components::interface::navigation::PelicanError;
use pelican_ui::components::avatar::{AvatarContent, AvatarIconStyle};
use pelican_ui::components::interface::general::{Bumper, Content, Header, Interface, Page};
use pelican_ui::plugin::PelicanUI;
use pelican_ui::theme::Theme;
use pelican_ui::components::interface::navigation::{AppPage, RootInfo};
use pelican_ui::page;

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
    async fn new(ctx: &mut Context) -> impl Drawable {
        // Create the first screen
        // let home = StartScreen::new(ctx);
        let home = RootInfo::icon("home", "Placeholder", |ctx: &mut Context | Box::new(StartScreen::new(ctx).ok().unwrap()) as Box<dyn AppPage>);
        // let ios_nav = ("boot", "IOS".to_string(), None, Some(Box::new(|ctx: &mut Context| Box::new(IOSScreen::new(ctx)) as Box<dyn AppPage>) as Box<dyn FnMut(&mut Context) -> Box<dyn AppPage>>));
        // let android_nav = ("cancel", "Android".to_string(), None, Some(Box::new(|ctx: &mut Context| Box::new(AndroidScreen::new(ctx)) as Box<dyn AppPage>) as Box<dyn FnMut(&mut Context) -> Box<dyn AppPage>>));
        // let navigation = (0usize, vec![android_nav], vec![ios_nav
        // ]);
        
        // Create the main interface with the first screen as the starting page
        Interface::new(ctx, (vec![home], None))
    }

    //provide a list of plugins used by the app
    fn plugins(ctx: &mut Context) -> Vec<Box<dyn Plugin>> {
        ctx.assets.include_assets(include_dir!("./assets/resources"));
        let theme = Theme::dark(&mut ctx.assets, Color::from_hex("#ff1f23", 255));
        vec![Box::new(PelicanUI::new(ctx, theme))]
    }
}

// Macro to start the application
start!(RampGUI);