use pelican_ui::{Component, Context, Plugins, Plugin, start, Application};
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Layout, SizeRequest, Area};
use pelican_ui::events::OnEvent;
use std::collections::BTreeMap;
use pelican_ui_std::AppPage;
use pelican_ui_std::components::interface::general::{Bumper, Interface, Page, Content, Header, HeaderIcon};
use pelican_ui_std::layout::{Stack, Offset};
use pelican_ui_std::components::{Text, TextInput, TextStyle, Icon, ExpandableText,};
use pelican_ui_std::components::button::{Button, ButtonStyle, ButtonWidth, ButtonState, ButtonSize, IconButton};
use pelican_ui_std::events::NavigateEvent;

#[derive(Debug, Clone)]
pub struct ErrorComponent;

impl OnEvent for ErrorComponent {}

impl ErrorComponent {
    pub fn new() -> Self {
        ErrorComponent
    }
}

#[derive(Debug, Component)]
pub struct ErrorScreen(Stack, Page);

// Implement event handling for ErrorScreen (empty for now)
impl OnEvent for ErrorScreen {}

// Implement the AppPage trait for navigation and UI behavior   
impl AppPage for ErrorScreen {
    fn has_nav(&self) -> bool { false }

    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
       Err(self)
    }
}

impl ErrorScreen {
    pub fn new(ctx: &mut Context) -> Self {
        let error_cache:String = ctx.state().get_named_mut::<String>("error_cache").unwrap().to_string();
        // Create a header for the page
        let header = Header::stack(
            ctx,
            None,
            "Error", 
            None
        );

        let font_size = ctx.theme.fonts.size;

        // Create the main heading text
        let text = Text::new(
            ctx,
            "An error has occurred...",
            TextStyle::Heading,
            font_size.h2,
            Align::Center
        );

        // Create the error text
        let text = Text::new(
            ctx,
            &error_cache,
            TextStyle::Error,
            font_size.md,
            Align::Center
        );

        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(text)]
        );


        ErrorScreen(Stack::default(), Page::new(Some(header), content, None))
    }
}