use pelican_ui::{Component, Context, Plugins, Plugin, start, Application};
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Layout, SizeRequest, Area};
use pelican_ui::events::OnEvent;
use std::collections::BTreeMap;
use pelican_ui_std::AppPage;
use pelican_ui_std::components::interface::general::{Interface, Page, Content, Header};
use pelican_ui_std::layout::{Stack, Offset};
use pelican_ui_std::components::{Text, TextStyle, Icon, ExpandableText};

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

// Define the main application struct. This is our entry point type.
pub struct MyApp;

// Implement the Services trait for MyApp
impl Services for MyApp {
    // Provide a list of services used by the app. Here, it's empty.
    fn services() -> ServiceList {
        ServiceList(BTreeMap::new())
    }
}

// Implement the Plugins trait for MyApp
impl Plugins for MyApp {
    // Provide a list of plugins used by the app. Currently, there are none.
    fn plugins(_ctx: &mut Context) -> Vec<Box<dyn Plugin>> { vec![] }
}

// Implement the Application trait for MyApp
impl Application for MyApp {
    // Asynchronously create the main drawable UI component
    async fn new(ctx: &mut Context) -> Box<dyn Drawable> {
        // Create the first screen
        let home = FirstScreen::new(ctx);
        // Create the main interface with the first screen as the starting page
        let interface = Interface::new(ctx, Box::new(home), None, None);
        // Return the interface wrapped in a Box
        Box::new(interface)
    }
}

// Macro to start the application
start!(MyApp);

// Define the first screen of the app
#[derive(Debug, Component)]
pub struct FirstScreen(Stack, Page);

// Implement event handling for FirstScreen (empty for now)
impl OnEvent for FirstScreen {}

// Implement the AppPage trait for navigation and UI behavior
impl AppPage for FirstScreen {
    // This screen does not have a navigation bar
    fn has_nav(&self) -> bool { false }

    // Handle page navigation. Always returns Err(self) because this page cannot navigate.
    fn navigate(self: Box<Self>, _ctx: &mut Context, _index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        Err(self)
    }
}

impl FirstScreen {
    pub fn new(ctx: &mut Context) -> Self {
        // Create a header for the page
        let header = Header::home(
            // The majority of UI components will require the app context.
            ctx,
            // The text on this header will say "My Screen"
            "Ramp", 
            // There will not be an icon button on this header
            None
        );

        let font_size = ctx.theme.fonts.size;
        let color = ctx.theme.colors.text.heading;

        // Create an icon element
        let icon = Icon::new(
            // This element requires the app context
            ctx, 
            // We choose the "pelican_ui" icon
            "pelican_ui", 
            // The color of the icon
            color, 
            // The size of the icon. Icons are always square.
            128.0
        );

        // Create the main heading text
        let text = Text::new(
            ctx,
            // This text will say "Hello World!"
            "Hello World!",
            // The style of this text will be heading
            TextStyle::Heading,
            // The size will be h2
            font_size.h2,
            // The text alignment
            Align::Center
        );

        // Create subtext.
        let subtext = ExpandableText::new(
            ctx,
            "First project loaded successfully.",
            // This text will have primary text style.
            TextStyle::Primary,
            // Medium font size
            font_size.md,
            // Center the text
            Align::Center,
            // No max lines
            None
        );

        // Combine icon, heading, and subtext into page content
        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(icon), Box::new(text), Box::new(subtext)]
        );

        // Return the FirstScreen with a default Stack and a 
        // new Page containinhg our header, content, and no bumper.
        FirstScreen(Stack::default(), Page::new(Some(header), content, None))
    }
}