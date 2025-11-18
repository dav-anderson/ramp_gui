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
use crate::ramp::session::{Session};
use crate::ramp::core::{new_project};

use serde::{Serialize, Deserialize};

#[derive(Debug, Component)]
pub struct AndroidScreen(Stack, Page);

impl OnEvent for AndroidScreen {}

impl AppPage for AndroidScreen {}

impl AndroidScreen {
    pub fn new(ctx: &mut Context) -> Result<Self, String> {
        //page header
        let header = Header::home(
            //app context
            ctx,
            //header string
            "<Project_name> Android",
            Some(("close", Box::new(|ctx: &mut Context| {
                let page = Box::new(StartScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))})
            ))
        );

        //main heading text
        let text = ExpandableText::new(
            ctx,
            //content
            "Build for Android",
            //Size
            TextSize::H2,
            //style
            TextStyle::Heading,
            //alignment
            Align::Center,
            None
        );

        let explainer = ExpandableText::new(
            ctx,
            //content
            "Ramp supports streaming install via usb tether. Please connect an Android device with developer debugging enabled.",
            //Size
            TextSize::H3,
            //style
            TextStyle::Primary,
            //alignment
            Align::Center,
            None
        );

        let tether = ExpandableText::new(
            ctx,
            //content
            "Device Connection Status: Not Ready",
            //Size
            TextSize::H4,
            //style
            TextStyle::Secondary,
            //alignment
            Align::Center,
            None
        );

        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(text), Box::new(explainer), Box::new(tether)]
        );

        let bumper = Bumper::home(
            ctx, 
            ("Debug", |ctx: &mut Context| {
                println!("Debug build")
            }), 
            Some(
                ("Release", Box::new(|ctx: &mut Context| {
                    println!("release build")
                })))
        );

        Ok(Self(Stack::default(), Page::new(header, content, Some(bumper))))
    }
}