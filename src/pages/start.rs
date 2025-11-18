
use pelican_ui::drawable::{Drawable, Color, Align};
use pelican_ui::{include_dir, drawables, Component, Context, Application, Plugin};
use pelican_ui::layouts::{Offset, Stack};
use pelican_ui::events::{OnEvent, Event, TickEvent};
use pelican_ui::components::button::PrimaryButton;
use pelican_ui::components::{ExpandableText, Icon, Text, TextStyle, TextSize};
// use pelican_ui::components::interface::navigation::PelicanError;
use pelican_ui::components::interface::general::{Bumper, Content, Header, Interface, Page};
use pelican_ui::plugin::PelicanUI;
use pelican_ui::components::interface::navigation::{AppPage, RootInfo, NavigationEvent};
use pelican_ui::interactions::Button;
use crate::pages::new::NewProjectScreen;
use crate::pages::load::LoadProjectScreen;
use crate::ramp::session::{Session};

use serde::{Serialize, Deserialize};

// Define the first screen of the app
#[derive(Debug, Component)]
pub struct StartScreen(Stack, Page);

// Implement event handling for StartScreen (empty for now)
impl OnEvent for StartScreen {}

// Implement the AppPage trait for navigation and UI behavior
impl AppPage for StartScreen {}

impl StartScreen {
    pub fn new(ctx: &mut Context) -> Result<Self, String> {
        if ctx.state().get_named_mut::<Session>("session").is_none(){
            //create the session state if it doesn't exist
            println!("creating session token");
            let mut session = match Session::new() {
                Ok(s) => s,
                Err(e) => {
                    // ctx.trigger_event(NavigateEvent(0));
                    Session::default()
                }
            };
            println!("blank session token: {:?}", session);
            println!("populating session token");
            match session.get_all_paths() { //update the session state from config file
                Ok(())=> {},
                Err(e) => {
                    // ctx.trigger_event(NavigateEvent(0));
                }
            };
            println!("populated session token: {:?}", session);
            // println!("saving session token");
            // ctx.state().set_named("session".to_string(), session);
        }
        //page header
        let header = Header::home(
            //app context
            ctx,
            //header string
            "Ramp", 
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

        //main heading text
        let text = ExpandableText::new(
            ctx,
            //content
            "Welcome to Ramp",
            //Size
            TextSize::H1,
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
            vec![Box::new(icon), Box::new(text)]
        );

        // let bumper = Bumper::home(ctx, ("new"), Some(("load")));
        let bumper = Bumper::home(
            ctx, 
            ("new", |ctx: &mut Context| {
                let page = Box::new(NewProjectScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))
            }), 
            Some(
                ("load", Box::new(|ctx: &mut Context| {
                    let page = Box::new(LoadProjectScreen::new(ctx).unwrap());
                    ctx.trigger_event(NavigationEvent::Push(Some(page)))
                })))
        );


        // Return the StartScreen with a default Stack
        Ok(Self(Stack::default(), Page::new(header, content, Some(bumper))))
    }
}
