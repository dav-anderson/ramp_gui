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
pub struct DebugScreen(Stack, Page);

impl OnEvent for DebugScreen {}

impl AppPage for DebugScreen {}

impl DebugScreen {
    pub fn new(ctx: &mut Context) -> Result<Self, String> {
        // let list_items: Vec<ListItem> = devices.devices_list.iter().map(|(name, date)| {
        //     ListItem::new(
        //         ctx,
        //         Some(AvatarContent::Icon("explore".to_string(), AvatarIconStyle::Primary)),
        //         ListItemInfoLeft::new(name, &format!("Created: {}", date), None, None),
        //         None,
        //         None,
        //         None,
        //         |ctx: &mut Context| {
        //             // session.update_current_project(project);
        //             let page = Box::new(DebugScreen::new(ctx).unwrap());
        //             ctx.trigger_event(NavigationEvent::Push(Some(page)))
        //         }
        //     )
        // }).collect();
        //page header
        let header = Header::home(
            //app context
            ctx,
            //header string
            "<Project_name> Debug",
            None
        );

        //main heading text
        let text = ExpandableText::new(
            ctx,
            //content
            "Click on a device to run your app",
            //Size
            TextSize::H2,
            //style
            TextStyle::Heading,
            //alignment
            Align::Center,
            None
        );

        // let list = drawables![ListItemGroup::new(list_items)];

        // Combine icon, heading, and subtext into page content
        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(text)] //, Box::new(list)
        );
        Ok(Self(Stack::default(), Page::new(header, content, None)))
    }
}