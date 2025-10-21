use pelican_ui::{Component, Context, Plugins, Plugin, start, Application};
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Layout, SizeRequest, Area};
use pelican_ui::events::{OnEvent};
use std::collections::BTreeMap;
use pelican_ui_std::AppPage;
use pelican_ui_std::components::interface::general::{Interface, Page, Content, Header};
use pelican_ui_std::layout::{Stack, Offset};
use pelican_ui_std::components::{Text, TextStyle, Icon, ExpandableText,};
use pelican_ui_std::components::button::{Button, ButtonStyle, ButtonWidth, ButtonState, ButtonSize};
use pelican_ui_std::events::NavigateEvent;
use crate::pages::new::NewProjectScreen;

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
        let home = StartScreen::new(ctx);
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
pub struct StartScreen(Stack, Page);

// Implement event handling for StartScreen (empty for now)
impl OnEvent for StartScreen {}

// Implement the AppPage trait for navigation and UI behavior
impl AppPage for StartScreen {
    // This screen does not have a navigation bar
    fn has_nav(&self) -> bool { false }

    // Handle page navigation. Always returns Err(self) because this page cannot navigate.
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        match index {
            0 => Ok(Box::new(NewProjectScreen::new(ctx))),
            _ => Err(self),
        }
        
    }
}

impl StartScreen {
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
            "Welcome to Ramp",
            // The style of this text will be heading
            TextStyle::Heading,
            // The size will be h2
            font_size.h2,
            // The text alignment
            Align::Center
        );

        // Create subtext.
        let new_project = Button::primary(
            ctx,
            "New",
            |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0))
            //on_click
        );

        // Combine icon, heading, and subtext into page content
        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(icon), Box::new(text), Box::new(new_project)]
        );

        // Return the StartScreen with a default Stack and a 
        // new Page containinhg our header, content, and no bumper.
        StartScreen(Stack::default(), Page::new(Some(header), content, None))
    }
}