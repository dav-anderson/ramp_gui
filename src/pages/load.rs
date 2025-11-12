use pelican_ui::drawable::{Drawable, Color, Align};
use pelican_ui::{include_dir, drawables, Component, Context, Application, Plugin};
use pelican_ui::layouts::{Offset, Stack};
use pelican_ui::events::{OnEvent, Event, TickEvent};
use pelican_ui::components::button::PrimaryButton;
use pelican_ui::components::{ExpandableText, Icon, RadioSelector, Text, TextStyle, TextSize, TextInput};
use pelican_ui::components::interface::general::{Bumper, Content, Header, Interface, Page};
use pelican_ui::plugin::PelicanUI;
use pelican_ui::components::interface::navigation::{AppPage, RootInfo, NavigationEvent};
use pelican_ui::interactions::Button;
use pelican_ui::utils::Callback;
use std::path::Path;
use std::fs;
use crate::pages::start::StartScreen;
use crate::pages::dashboard::DashboardScreen;
use crate::ramp::session::{Session};
use crate::ramp::core::{new_project};

use serde::{Serialize, Deserialize};

// #[derive (Debug)]
// pub struct project_paths;

// impl project_paths {
//     fn get_items_at_path(path: &str) -> Result<Vec<(&str, &str, Callback)>, std::io::Error> {
//         let path = Path::new(path);
//         if !path.is_dir() {
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::InvalidInput,
//                 "Path is not a directory",
//             ));
//         }
    
//         let mut items = Vec::new();
//         for entry in fs::read_dir(path)? {
//             let entry = entry?;
//             let name = entry.file_name().into_string().map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
//             let name_first = name.clone();
//             let name_second = "ramp project";
//             let name_for_callback = name.clone();
    
//             let callback = move |_| {
//                 println!("Selected {}", name_for_callback)
//             };
//             items.push((name_first, name_second, callback));
//         }
//         Ok(items)
//     }
// }

//define the page
#[derive(Debug, Component)]
pub struct LoadProjectScreen(Stack, Page);

// Implement event handling for New Project Screen
impl OnEvent for LoadProjectScreen {}

// Implement the AppPage trait for navigation and UI behavior
impl AppPage for LoadProjectScreen {}

impl LoadProjectScreen {
    pub fn new(ctx: &mut Context) -> Result<Self, String> {
        let session = ctx.state().get_named_mut::<Session>("session");

        let projects_list:Vec<(&str, &str, Callback)> = vec![
            ("test", "description", Box::new(move |_| {println!("Selected test") })), 
            ("test2", "description2", Box::new(move |_| {println!("Selected test2")})),
            ("test3", "description3", Box::new(move |_| {println!("Selected test3")})),
        ];
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
        let list_select = RadioSelector::new(
            //context
            ctx,
            projects_list.len(),
            projects_list
            //index: usize,
            //items: Vec<(&str, &str, Callback)>,
        );

        // Combine icon, heading, and subtext into page content
        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(text), Box::new(list_select)]
        );

        //
        let bumper = Bumper::home(
            ctx, 
            ("Load", |ctx: &mut Context| {
                let page = Box::new(DashboardScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))
            }), 
            Some(
                ("Delete", Box::new(|ctx: &mut Context| {
                    println!("delete entry")
                })))
        );

        // Return the StartScreen with a default Stack
        Ok(Self(Stack::default(), Page::new(header, content, Some(bumper))))
    }
}