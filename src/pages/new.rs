use pelican_ui::{Component, Context, Plugins, Plugin, start, Application};
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Layout, SizeRequest, Area};
use pelican_ui::events::OnEvent;
use std::collections::BTreeMap;
use pelican_ui_std::AppPage;
use pelican_ui_std::components::interface::general::{Interface, Page, Content, Header};
use pelican_ui_std::layout::{Stack, Offset};
use pelican_ui_std::components::{Text, TextStyle, Icon, ExpandableText,};
use pelican_ui_std::components::button::{Button, ButtonStyle, ButtonWidth, ButtonState, ButtonSize};


// Define the first screen of the app
#[derive(Debug, Component)]
pub struct NewProjectScreen(Stack, Page);

// Implement event handling for NewProjectScreen (empty for now)
impl OnEvent for NewProjectScreen {}

// Implement the AppPage trait for navigation and UI behavior   
impl AppPage for NewProjectScreen {
    // This screen does not have a navigation bar
    fn has_nav(&self) -> bool { false }

    // Handle page navigation. Always returns Err(self) because this page cannot navigate.
    fn navigate(self: Box<Self>, _ctx: &mut Context, _index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        Err(self)
    }
}

impl NewProjectScreen {
    pub fn new(ctx: &mut Context) -> Self {
        // Create a header for the page
        let header = Header::home(
            // The majority of UI components will require the app context.
            ctx,
            // The text on this header will say "My Screen"
            "Create New Project", 
            // There will not be an icon button on this header
            None
        );

        let font_size = ctx.theme.fonts.size;

        // Create the main heading text
        let text = Text::new(
            ctx,
            // This text will say "Hello World!"
            "Give your project a name",
            // The style of this text will be heading
            TextStyle::Heading,
            // The size will be h2
            font_size.h2,
            // The text alignment
            Align::Center
        );

        // Create subtext.
        // let new_button = Button::new(
        //     ctx,
        //     "New",
        //     |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0))
        //     //on_click
        // );

        // Combine icon, heading, and subtext into page content
        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(text)]
        );

        // Return the NewProjectScreen with a default Stack and a 
        // new Page containinhg our header, content, and no bumper.
        NewProjectScreen(Stack::default(), Page::new(Some(header), content, None))
    }
}