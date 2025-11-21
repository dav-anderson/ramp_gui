
use pelican_ui::drawable::{Drawable, Color, Align};
use pelican_ui::{include_dir, drawables, Component, Context, Application, Plugin};
use pelican_ui::layouts::{Offset, Stack};
use pelican_ui::events::{OnEvent, Event, TickEvent};
use pelican_ui::components::button::PrimaryButton;
use pelican_ui::components::{ExpandableText, Icon, Text, TextStyle, TextSize};
// use pelican_ui::components::interface::navigation::PelicanError;
use pelican_ui::components::interface::general::{Bumper, Content, Header, Interface, Page};
use pelican_ui::components::list_item::{ListItem, ListItemGroup, ListItemInfoLeft};
use pelican_ui::components::avatar::{AvatarContent, AvatarIconStyle};
use pelican_ui::plugin::PelicanUI;
use pelican_ui::components::interface::navigation::{AppPage, RootInfo, NavigationEvent};
use pelican_ui::interactions::Button;
use std::io;
use std::path::Path;
use std::fs;
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use crate::pages::new::NewProjectScreen;
use crate::pages::dashboard::DashboardScreen;
use crate::ramp::session::{Session};

use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct Projects{
    projects_list: Vec<(String, String)>
}

impl Projects {
    pub fn new() -> Self{
        Self {projects_list: vec![]}
    }

    pub fn retrieve(&mut self, path: &str) -> io::Result<()>{
        println!("populating projects vector");
        let path = Path::new(path);
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                let metadata = fs::metadata(entry.path())?;
                let created: SystemTime = metadata.created()?;
                let created_dt: DateTime<Utc> = created.into();
                let created_str = created_dt.format("%Y-%m-%d").to_string();
                self.projects_list.push((name, created_str));   
            }
        }
        Ok(())
    }
}

// Define the first screen of the app
#[derive(Debug, Component)]
pub struct StartScreen(Stack, Page);

// Implement event handling for StartScreen (empty for now)
impl OnEvent for StartScreen {}

// Implement the AppPage trait for navigation and UI behavior
impl AppPage for StartScreen {}

impl StartScreen {
    pub fn new(ctx: &mut Context) -> Result<Self, String> {
        //populate session state
        let mut session = Session::default();
        if ctx.state().get_named_mut::<Session>("session").is_none(){
            //create the session state if it doesn't exist
            println!("creating session token");
            session = match Session::new() {
                Ok(s) => s,
                Err(e) => {
                    println!("****ERROR POPULATING SESSION TOKEN********");
                    Session::default()
                }
            };
            println!("blank session token: {:?}", session);
            println!("populating session token");
            match session.get_all_paths() { //update the session state from config file
                Ok(())=> {},
                Err(e) => {
                    println!("******ERROR POPULATING SESSION TOKEN******");
                }
            };
            println!("populated session token: {:?}", session);
            // println!("saving session token");
            // ctx.state().set_named("session".to_string(), session);
        }
        let mut projects = Projects::new();
        projects.retrieve(session.projects_path.as_ref().unwrap());
        println!("projects vector: {:?}", projects);
        //TODO onclick should update the current project string in session state
        //TODO populate this list with items from the project dir, create dynamically
        let list_items: Vec<ListItem> = projects.projects_list.iter().map(|(name, date)| {
            ListItem::new(
                ctx,
                Some(AvatarContent::Icon("explore".to_string(), AvatarIconStyle::Primary)),
                ListItemInfoLeft::new(name, &format!("Created: {}", date), None, None),
                None,
                None,
                None,
                |ctx: &mut Context| {
                    // session.update_current_project(project);
                    let page = Box::new(DashboardScreen::new(ctx).unwrap());
                    ctx.trigger_event(NavigationEvent::Push(Some(page)))
                }
            )
        }).collect();

        //page header
        let header = Header::home(
            //app context
            ctx,
            //header string
            "Welcome to Ramp", 
            //No icon button
            None
        );

        //icon element
        let icon = Icon::new(
            //app context
            ctx, 
            //icon
            "pelican", 
            //icon color
            None,
            //icon size
            128.0
        );        

        let list = drawables![ListItemGroup::new(list_items)];
        //insert list item to load projects here

        //main heading text
        // let list_select = RadioSelector::new(
        //     //context
        //     ctx,
        //     projects_list.len(),
        //     projects_list
        //     //index: usize,
        //     //items: Vec<(&str, &str, Callback)>,
        // );

        // Combine icon, heading, and subtext into page content
        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            list
        );

        // let bumper = Bumper::home(ctx, ("new"), Some(("load")));
        let bumper = Bumper::home(
            ctx, 
            ("new project", |ctx: &mut Context| {
                let page = Box::new(NewProjectScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))
            }), 
            None
        );


        // Return the StartScreen with a default Stack
        Ok(Self(Stack::default(), Page::new(header, content, Some(bumper))))
    }
}
