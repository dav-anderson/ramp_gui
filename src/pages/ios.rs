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


// #[derive(Debug, Component)]
// pub struct IOSScreen(Stack, Page);

// // Implement event handling for IOSScreen
// impl OnEvent for IOSScreen {}

// // Implement the AppPage trait for navigation and UI behavior   
// impl AppPage for IOSScreen {
//     fn has_nav(&self) -> bool { true }

//     fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
//         match index {
//             0 => Ok(Box::new(ErrorScreen::new(ctx))),
//             1 => Ok(Box::new(StartScreen::new(ctx))),
//             _ => Err(self),
//         }
//     }
// }

// impl IOSScreen {
//     pub fn new(ctx: &mut Context) -> Self {
//         let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)));
//         // Create a header for the page
//         let header = Header::stack(
//             ctx,
//             Some(back),
//             "<ProjectName> IOS", 
//             None
//         );

//         let font_size = ctx.theme.fonts.size;

//         // Create the main heading text
//         let text = Text::new(
//             ctx,
//             "WIP build ios tools go here",
//             TextStyle::Heading,
//             font_size.h2,
//             Align::Center
//         );

//         // Combine icon, heading, and subtext into page content
//         let content = Content::new(
//             ctx,
//             // Vertically center items
//             Offset::Center,
//             // All items must be boxed as Box<dyn Drawable>
//             vec![Box::new(text)]
//         );

//          // Create a new project.
//          let debug_btn = Button::primary(
//             ctx,
//             "Build Debug",
//             //on_click
//             |ctx: &mut Context| println!("Debug button")
//         );
        
//         // Create a new project.
//         let release_btn = Button::primary(
//             ctx,
//             "Build Release",
//             //on_click
//             |ctx: &mut Context| println!("Release button")
//         );

//         let bumper = Bumper::double_button(ctx, debug_btn, release_btn);

//         IOSScreen(Stack::default(), Page::new(Some(header), content, Some(bumper)))
//     }
// }