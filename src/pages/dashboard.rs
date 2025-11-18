use pelican_ui::drawable::{Drawable, Color, Align};
use pelican_ui::{include_dir, drawables, Component, Context, Application, Plugin};
use pelican_ui::layouts::{Offset, Stack};
use pelican_ui::events::{OnEvent, Event, TickEvent};
use pelican_ui::components::button::PrimaryButton;
use pelican_ui::components::{ExpandableText, Icon, Text, TextStyle, TextSize, TextInput};
use pelican_ui::components::interface::general::{Bumper, Content, Header, Interface, Page};
use pelican_ui::plugin::PelicanUI;
use pelican_ui::components::interface::navigation::{AppPage, RootInfo, NavigationEvent};
use pelican_ui::pages::Error;
use pelican_ui::interactions::Button;
use crate::pages::start::StartScreen;
use pelican_ui::components::avatar::{Avatar, AvatarContent, AvatarIconStyle, AvatarSize};
use crate::ramp::session::{Session};
use crate::ramp::core::{new_project};


use serde::{Serialize, Deserialize};

#[derive(Debug, Component)]
pub struct DashboardScreen(Stack, Page);

impl OnEvent for DashboardScreen {}

impl AppPage for DashboardScreen {
}

impl DashboardScreen {
    pub fn new(ctx: &mut Context) -> Result<Self, Error> {
        let project_loaded: bool = false;
        if !project_loaded {
            println!("*******PROJECT NOT LOADED********");
            let start = Box::new(StartScreen::new(ctx).unwrap());
            ctx.trigger_event(NavigationEvent::Push(Some(start)))
        }
        //page header
        let header = Header::home(
            //app context
            ctx,
            //header string
            "General",
            Some(("close", Box::new(|ctx: &mut Context| {
                let page = Box::new(StartScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))})
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

        // let avatar = ctx.assets.load_image("/Users/davidanderson/ramp_gui/assets/resources/icons/icon.png").unwrap();

        //App icon goes here
        let app_icon = Avatar::new(
            ctx,
            // AvatarContent::Image(avatar),
            AvatarContent::Icon("car".to_string(), AvatarIconStyle::Primary),
            Some(("edit", AvatarIconStyle::Primary)),
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

        // Combine icon, heading, and subtext into page content
        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(text), Box::new(app_icon), Box::new(name_input)]
        );

        let bumper = Bumper::home(
            ctx, 
            ("Run", |ctx: &mut Context| {
                println!("Run Build Locally")
            }), 
            Some(
                ("Cargo Clean", Box::new(|ctx: &mut Context| {
                    println!("Cleaning Cargo Cache")
                })))
        );

        Ok(Self(Stack::default(), Page::new(header, content, Some(bumper))))
    }
}