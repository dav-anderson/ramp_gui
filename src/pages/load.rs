use pelican_ui::{Component, Context, Plugins, Plugin, start, Application};
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Layout, SizeRequest, Area};
use pelican_ui::events::OnEvent;
use std::collections::BTreeMap;
use pelican_ui_std::AppPage;
use pelican_ui_std::components::interface::general::{Bumper, Interface, Page, Content, Header};
use pelican_ui_std::layout::{Stack, Offset};
use pelican_ui_std::components::{Text, TextStyle, Icon, ExpandableText,};
use pelican_ui_std::components::button::{Button, ButtonStyle, ButtonWidth, ButtonState, ButtonSize, IconButton};
use pelican_ui_std::events::NavigateEvent;
use crate::pages::start::StartScreen;
use crate::pages::error::ErrorScreen;
use crate::pages::dashboard::DashboardScreen;

#[derive(Debug, Component)]
pub struct LoadProjectScreen(Stack, Page);

// Implement event handling for LoadProjectScreen
impl OnEvent for LoadProjectScreen {}

// Implement the AppPage trait for navigation and UI behavior   
impl AppPage for LoadProjectScreen {
    fn has_nav(&self) -> bool { false }

    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        match index {
            0 => Ok(Box::new(ErrorScreen::new(ctx))),
            1 => Ok(Box::new(StartScreen::new(ctx))),
            2 => Ok(Box::new(DashboardScreen::new(ctx))),
            
            _ => Err(self),
        }
    }
}

impl LoadProjectScreen {
    pub fn new(ctx: &mut Context) -> Self {
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)));
        // Create a header for the page
        let header = Header::stack(
            ctx,
            Some(back),
            "Load Existing Project", 
            None
        );

        let font_size = ctx.theme.fonts.size;

        // Create the main heading text
        let text = Text::new(
            ctx,
            "selectable list goes here",
            TextStyle::Heading,
            font_size.h2,
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

        // Create a new project.
        let load_btn = Button::primary(
            ctx,
            "Load",
            //on_click
            |ctx: &mut Context| ctx.trigger_event(NavigateEvent(2))
        );
        
        // Create a new project.
        let delete_btn = Button::primary(
            ctx,
            "Delete",
            //on_click
            |ctx: &mut Context| println!("Delete button")
        );

        let bumper = Bumper::double_button(ctx, load_btn, delete_btn);

        LoadProjectScreen(Stack::default(), Page::new(Some(header), content, Some(bumper)))
    }
}