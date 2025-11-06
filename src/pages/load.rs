use pelican_ui::drawable::{Drawable, Color, Align};
use pelican_ui::{include_dir, drawables, Component, Context, Application, Plugin};
use pelican_ui::layouts::{Offset, Stack};
use pelican_ui::events::{OnEvent, Event, TickEvent};
use pelican_ui::components::button::PrimaryButton;
use pelican_ui::components::{ExpandableText, Icon, Text, TextStyle, TextSize, TextInput};
use pelican_ui::components::interface::navigation::PelicanError;
use pelican_ui::components::interface::general::{Bumper, Content, Header, Interface, Page};
use pelican_ui::plugin::PelicanUI;
use pelican_ui::components::interface::navigation::{AppPage, RootInfo, NavigateEvent};
use pelican_ui::interactions::Button;
use pelican_ui::page;
use crate::pages::start::StartScreen;
// use crate::pages::new::DashboardScreen;
use crate::ramp::session::{Session};
use crate::ramp::core::{new_project};

use serde::{Serialize, Deserialize};

//define the page
#[derive(Debug, Component)]
pub struct LoadProjectScreen(Stack, Page);

// Implement event handling for New Project Screen
impl OnEvent for LoadProjectScreen {}

// Implement the AppPage trait for navigation and UI behavior
impl AppPage for LoadProjectScreen {
    // This screen does not have a navigation bar
    fn has_navigator(&self) -> bool { false }

    // Handle page navigation. Always returns Err(self) because this page cannot navigate.
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, PelicanError> {
        match index {
            0 => page!(Box::new(StartScreen::new(ctx)), self),
            // 1 => page!(DashboardScreen::new(ctx), self),
            _ => Err(PelicanError::InvalidPage(Some(self))),
        }        
    }
}

impl LoadProjectScreen {
    pub fn new(ctx: &mut Context) -> Result<Self, String> {
        //page header
        let header = Header::stack(
            //app context
            ctx,
            //header string
            "Ramp", 
        );

        //main heading text
        let text = ExpandableText::new(
            ctx,
            //content
            "Load your Project",
            //Size
            TextSize::H2,
            //style
            TextStyle::Heading,
            //alignment
            Align::Center,
            None
        );

        //main heading text
        let list_select = ExpandableText::new(
            ctx,
            //content
            "project select component goes here",
            //Size
            TextSize::H4,
            //style
            TextStyle::Heading,
            //alignment
            Align::Center,
            None
        );

        // Combine icon, heading, and subtext into page content
        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(text), Box::new(list_select)]
        );

        let bumper = Bumper::home(ctx, "load", None);

        // Return the StartScreen with a default Stack
        Ok(Self(Stack::default(), Page::new(header, content, Some(bumper))))
    }
}







// use pelican::{Component, Context, Plugins, Plugin, start, Application};
// use pelican::drawable::{Drawable, Component, Align};
// use pelican::layout::{Layout, SizeRequest, Area};
// use pelican::events::OnEvent;
// use std::collections::BTreeMap;
// use pelican::AppPage;
// use pelican::components::interface::general::{Bumper, Interface, Page, Content, Header};
// use pelican::layout::{Stack, Offset};
// use pelican::components::{Text, TextStyle, Icon, ExpandableText,};
// use pelican::components::button::{Button, ButtonStyle, ButtonWidth, ButtonState, ButtonSize, IconButton};
// use pelican::events::NavigateEvent;
// use crate::pages::start::StartScreen;
// use crate::pages::error::ErrorScreen;
// use crate::pages::dashboard::DashboardScreen;

// #[derive(Debug, Component)]
// pub struct LoadProjectScreen(Stack, Page);

// // Implement event handling for LoadProjectScreen
// impl OnEvent for LoadProjectScreen {}

// // Implement the AppPage trait for navigation and UI behavior   
// impl AppPage for LoadProjectScreen {
//     fn has_nav(&self) -> bool { false }

//     fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
//         match index {
//             0 => Ok(Box::new(ErrorScreen::new(ctx))),
//             1 => Ok(Box::new(StartScreen::new(ctx))),
//             2 => Ok(Box::new(DashboardScreen::new(ctx))),
            
//             _ => Err(self),
//         }
//     }
// }

// impl LoadProjectScreen {
//     pub fn new(ctx: &mut Context) -> Self {
//         let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)));
//         // Create a header for the page
//         let header = Header::stack(
//             ctx,
//             Some(back),
//             "Load Existing Project", 
//             None
//         );

//         let font_size = ctx.theme.fonts.size;

//         // Create the main heading text
//         let text = Text::new(
//             ctx,
//             "selectable list goes here",
//             TextStyle::Heading,
//             font_size.h2,
//             Align::Center
//         );

//         // Create subtext.
//         // let new_button = Button::new(
//         //     ctx,
//         //     "New",
//         //     |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0))
//         //     //on_click
//         // );

//         // Combine icon, heading, and subtext into page content
//         let content = Content::new(
//             ctx,
//             // Vertically center items
//             Offset::Center,
//             // All items must be boxed as Box<dyn Drawable>
//             vec![Box::new(text)]
//         );

//         // Create a new project.
//         let load_btn = Button::primary(
//             ctx,
//             "Load",
//             //on_click
//             |ctx: &mut Context| ctx.trigger_event(NavigateEvent(2))
//         );
        
//         // Create a new project.
//         let delete_btn = Button::primary(
//             ctx,
//             "Delete",
//             //on_click
//             |ctx: &mut Context| println!("Delete button")
//         );

//         let bumper = Bumper::double_button(ctx, load_btn, delete_btn);

//         LoadProjectScreen(Stack::default(), Page::new(Some(header), content, Some(bumper)))
//     }
// }