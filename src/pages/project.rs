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
use pelican_ui::components::list_item::{ListItem, ListItemGroup, ListItemInfoLeft};
use pelican_ui::components::avatar::{AvatarContent, AvatarIconStyle};
use crate::pages::start::StartScreen;
use crate::ramp::session::{Session};
use crate::pages::settings::SettingsScreen;
use crate::pages::ios::IOSScreen;
use crate::pages::android::AndroidScreen;
use crate::pages::linux::LinuxScreen;
use crate::pages::macos::MacOSScreen;
use crate::pages::wasm::WASMScreen;
use crate::pages::windows::WindowsScreen;
use crate::ramp::core::{new_project};

use serde::{Serialize, Deserialize};

#[derive(Debug, Component)]
pub struct ProjectScreen(Stack, Page);

impl OnEvent for ProjectScreen {}

impl AppPage for ProjectScreen {}

impl ProjectScreen {
    pub fn new(ctx: &mut Context) -> Result<Self, String> {
        //page header
        let header = Header::home(
            //app context
            ctx,
            //header string
            "<Project_name>",
            None
        );

        let device_text = ExpandableText::new(
            ctx,
            //content
            "Run on a Device",
            //Size
            TextSize::H3,
            //style
            TextStyle::Heading,
            //alignment
            Align::Center,
            None
        );

        let local: ListItem = ListItem::new(
            ctx,
            Some(AvatarContent::Icon("monitor".to_string(), AvatarIconStyle::Primary)),
            ListItemInfoLeft::new("MacOS Desktop", "This Computer", None, None),
            None,
            None,
            None,
            |ctx: &mut Context| {
                println!("deploying to local desktop")
            }

        );
        let ios_usb: ListItem = ListItem::new(
            ctx,
            Some(AvatarContent::Icon("info".to_string(), AvatarIconStyle::Warning)),
            ListItemInfoLeft::new("IOS Device", "Click to Setup", None, None),
            None,
            None,
            None,
            |ctx: &mut Context| {
                println!("deploying to IOS")
            }

        );
        let android_usb: ListItem = ListItem::new(
            ctx,
            Some(AvatarContent::Icon("pelican_ui".to_string(), AvatarIconStyle::Secondary)),
            ListItemInfoLeft::new("Android Device", "No device connected", None, None),
            None,
            None,
            None,
            |ctx: &mut Context| {
                println!("deploying to android")
            }

        );

        let device_list = ListItemGroup::new(vec![local, ios_usb, android_usb]);

        let release_text = ExpandableText::new(
            ctx,
            //content
            "Build for Platform",
            //Size
            TextSize::H3,
            //style
            TextStyle::Heading,
            //alignment
            Align::Center,
            None
        );

        let ios: ListItem = ListItem::new(
            ctx,
            Some(AvatarContent::Icon("app_store".to_string(), AvatarIconStyle::Primary)),
            ListItemInfoLeft::new("IOS", "Configuration", None, None),
            None,
            None,
            None,
            |ctx: &mut Context| {
                println!("building for ios");
                let page = Box::new(IOSScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))
            }

        );
        let macos: ListItem = ListItem::new(
            ctx,
            Some(AvatarContent::Icon("app_store".to_string(), AvatarIconStyle::Primary)),
            ListItemInfoLeft::new("MacOS", "Configuration", None, None),
            None,
            None,
            None,
            |ctx: &mut Context| {
                println!("building for MacOS");
                let page = Box::new(MacOSScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))
            }

        );
        let android: ListItem = ListItem::new(
            ctx,
            Some(AvatarContent::Icon("play_store".to_string(), AvatarIconStyle::Primary)),
            ListItemInfoLeft::new("Android", "Configuration", None, None),
            None,
            None,
            None,
            |ctx: &mut Context| {
                println!("building for android");
                let page = Box::new(AndroidScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))
            }

        );
        let linux: ListItem = ListItem::new(
            ctx,
            Some(AvatarContent::Icon("monitor".to_string(), AvatarIconStyle::Primary)),
            ListItemInfoLeft::new("Linux", "Configuration", None, None),
            None,
            None,
            None,
            |ctx: &mut Context| {
                println!("building for Linux");
                let page = Box::new(LinuxScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))
            }

        );
        let windows: ListItem = ListItem::new(
            ctx,
            Some(AvatarContent::Icon("monitor".to_string(), AvatarIconStyle::Primary)),
            ListItemInfoLeft::new("Windows", "Configuration", None, None),
            None,
            None,
            None,
            |ctx: &mut Context| {
                println!("building for windows");
                let page = Box::new(WindowsScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))
            }

        );
        let wasm: ListItem = ListItem::new(
            ctx,
            Some(AvatarContent::Icon("pelican_ui".to_string(), AvatarIconStyle::Primary)),
            ListItemInfoLeft::new("Web", "Configuration", None, None),
            None,
            None,
            None,
            |ctx: &mut Context| {
                println!("building for wasm");
                let page = Box::new(WASMScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))
            }

        );

        let release_list = ListItemGroup::new(vec![ios, macos, android, linux, windows, wasm]);

        let output_list = drawables![device_text, device_list, release_text, release_list];

        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Start,
            // All items must be boxed as Box<dyn Drawable>
            output_list
        );
        Ok(Self(Stack::default(), Page::new(header, content, None)))
    }
}