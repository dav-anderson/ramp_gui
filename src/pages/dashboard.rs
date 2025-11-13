use pelican_ui::drawable::{Drawable, Color, Align};
use pelican_ui::{include_dir, drawables, Component, Context, Application, Plugin};
use pelican_ui::layouts::{Offset, Stack};
use pelican_ui::events::{OnEvent, Event, TickEvent};
use pelican_ui::components::button::PrimaryButton;
use pelican_ui::components::{ExpandableText, Icon, Text, TextStyle, TextSize, TextInput};
use pelican_ui::components::interface::general::{Bumper, Content, Header, Interface, Page};
use pelican_ui::plugin::PelicanUI;
use pelican_ui::components::interface::navigation::{AppPage, RootInfo, NavigationEvent};
use pelican_ui::interactions::Button;
use crate::pages::start::StartScreen;
use pelican_ui::components::avatar::{Avatar, AvatarContent, AvatarIconStyle, AvatarSize};
use crate::ramp::session::{Session};
use crate::ramp::core::{new_project};

use serde::{Serialize, Deserialize};

//define the page
#[derive(Debug, Component)]
pub struct DashboardScreen(Stack, Page);

// Implement event handling for New Project Screen
impl OnEvent for DashboardScreen {}

// Implement the AppPage trait for navigation and UI behavior
impl AppPage for DashboardScreen {}

impl DashboardScreen {
    pub fn new(ctx: &mut Context) -> Result<Self, String> {
        //page header
        let header = Header::home(
            //app context
            ctx,
            //header string
            "<Project_name> Dashboard",
            Some(("close", Box::new(|ctx: &mut Context| {
                ctx.trigger_event(NavigationEvent::Reset)})
            ))
        );

        //main heading text
        let text = ExpandableText::new(
            ctx,
            //content
            "<Project Name>",
            //Size
            TextSize::H2,
            //style
            TextStyle::Heading,
            //alignment
            Align::Center,
            None
        );

        //App icon goes here
        let app_icon = Avatar::new(
            ctx,
            AvatarContent::Icon("icon".to_string(), AvatarIconStyle::Primary),
            None,
            false,
            AvatarSize::Xl,
            None,
        );

        let mut name_input = TextInput::new(
            ctx,
            None,
            Some("Project Name"),
            Some("project_name"),
            None,
            None
        );

        let mut bundle_input = TextInput::new(
            ctx,
            None,
            Some("Apple App ID Bundle"),
            Some("Apple_App_ID_Bundle"),
            None,
            None
        );

        // Combine icon, heading, and subtext into page content
        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(text), Box::new(app_icon), Box::new(name_input), Box::new(bundle_input)]
        );

        // let bumper = Bumper::home(ctx, "create", None);

        // Return the StartScreen with a default Stack
        Ok(Self(Stack::default(), Page::new(header, content, None)))
    }
}



// use pelican::{Component, Context, Plugins, Plugin, start, Application};
// use pelican::drawable::{Drawable, Component, Align};
// use pelican::layout::{Layout, SizeRequest, Area};
// use pelican::events::OnEvent;
// use std::collections::BTreeMap;
// use pelican::AppPage;
// use pelican::components::interface::general::{Interface, Page, Content, Header};
// use pelican::layout::{Stack, Offset};
// use pelican::components::{Text, TextStyle, Icon, ExpandableText,};
// use pelican::components::button::{Button, ButtonStyle, ButtonWidth, ButtonState, ButtonSize, IconButton};
// use pelican::events::NavigateEvent;
// use crate::pages::start::StartScreen;
// use crate::pages::error::ErrorScreen;
// use crate::pages::ios::IOSScreen;
// use crate::pages::macos::MacOSScreen;
// use crate::pages::android::AndroidScreen;
// use crate::pages::windows::WindowsScreen;
// use crate::pages::linux::LinuxScreen;
// use crate::pages::wasm::WASMScreen;
// use crate::ramp::session::{Session};

// #[derive(Debug, Component)]
// pub struct DashboardScreen(Stack, Page);

// // Implement event handling for DashboardScreen
// impl OnEvent for DashboardScreen {}

// // Implement the AppPage trait for navigation and UI behavior   
// impl AppPage for DashboardScreen {
//     fn has_nav(&self) -> bool { true }

//     fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
//         match index {
//             0 => Ok(Box::new(ErrorScreen::new(ctx))),
//             1 => Ok(Box::new(StartScreen::new(ctx))),
//             2 => Ok(Box::new(IOSScreen::new(ctx))),
//             3 => Ok(Box::new(MacOSScreen::new(ctx))),
//             4 => Ok(Box::new(AndroidScreen::new(ctx))),
//             5 => Ok(Box::new(WindowsScreen::new(ctx))),
//             6 => Ok(Box::new(LinuxScreen::new(ctx))),
//             7 => Ok(Box::new(WASMScreen::new(ctx))),
//             _ => Err(self),
//         }
//     }
// }

// impl DashboardScreen {
//     pub fn new(ctx: &mut Context) -> Self {
//         println!("Arrived at dashboard");
//         let session = ctx.state().get_named_mut::<Session>("session").unwrap();
//         println!("Session token: {:?}", session);
//         let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)));
//         // Create a header for the page
//         let header = Header::stack(
//             ctx,
//             Some(back),
//             &format!("{} dashboard", session.current_project.as_ref().unwrap().to_string()), 
//             None
//         );

//         let font_size = ctx.theme.fonts.size;

//         // Create the main heading text
//         let text = Text::new(
//             ctx,
//             "WIP app dashboard goes here",
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

//         DashboardScreen(Stack::default(), Page::new(Some(header), content, None))
//     }
// }