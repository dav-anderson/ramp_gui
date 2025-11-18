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
pub struct IOSScreen(Stack, Page);

impl OnEvent for IOSScreen {}

impl AppPage for IOSScreen {}

impl IOSScreen {
    pub fn new(ctx: &mut Context) -> Result<Self, String> {
        //page header
        let header = Header::home(
            //app context
            ctx,
            //header string
            "<Project_name> IOS",
            Some(("close", Box::new(|ctx: &mut Context| {
                let page = Box::new(StartScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))})
            ))
        );

        //main heading text
        let text = ExpandableText::new(
            ctx,
            //content
            "Build for IOS",
            //Size
            TextSize::H2,
            //style
            TextStyle::Heading,
            //alignment
            Align::Center,
            None
        );

        let mut bundle_input = TextInput::new(
            ctx,
            None,
            Some("Apple App ID Bundle"),
            Some("App_ID_Bundle"),
            None,
            None
        );

        let explainer = ExpandableText::new(
            ctx,
            //content
            "Ramp supports streaming install via usb tether. Please connect an IOS device with developer debugging enabled. You will be required to provision the device.",
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

        let provision = ExpandableText::new(
            ctx,
            //content
            "Device Provision Status: Not Ready",
            //Size
            TextSize::H4,
            //style
            TextStyle::Secondary,
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
            vec![Box::new(text), Box::new(bundle_input), Box::new(explainer), Box::new(tether), Box::new(provision)]
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


        // Return a default Stack
        Ok(Self(Stack::default(), Page::new(header, content, Some(bumper))))
    }
}