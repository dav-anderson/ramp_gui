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
use crate::pages::start::StartScreen;
use crate::pages::dashboard::DashboardScreen;

#[derive(Debug, Component)]
pub struct NewProjectScreen(Stack, Page);

// Implement event handling for NewProjectScreen (empty for now)
impl OnEvent for NewProjectScreen {}

// Implement the AppPage trait for navigation and UI behavior   
impl AppPage for NewProjectScreen {
    fn has_nav(&self) -> bool { false }

    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        match index {
            0 => Ok(Box::new(StartScreen::new(ctx))),
            1 => Ok(Box::new(DashboardScreen::new(ctx))),
            _ => Err(self),
        }
    }
}

impl NewProjectScreen {
    pub fn new(ctx: &mut Context) -> Self {
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        // Create a header for the page
        let header = Header::stack(
            ctx,
            Some(back),
            "Create New Project", 
            None
        );

        let font_size = ctx.theme.fonts.size;

        // // Create the main heading text
        // let text = Text::new(
        //     ctx,
        //     "Give your project a name",
        //     TextStyle::Heading,
        //     font_size.h2,
        //     Align::Center
        // );

        // Create name input.
        let mut name_input = TextInput::new(
            ctx,
            None,
            Some("Give Your Project a Name"),
            "Project name...",
            Some("This name will be applied across the app"),
            TextInput::NO_ICON,
            false
        );

        let create_button = Button::primary(
            ctx,
            "Create",
            //on_click
            |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1))
        );

        let bumper = Bumper::single_button(ctx, create_button);

        // Combine icon, heading, and subtext into page content
        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(name_input)]
        );

        NewProjectScreen(Stack::default(), Page::new(Some(header), content, Some(bumper)))
    }
}