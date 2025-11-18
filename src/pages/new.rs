
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
use crate::pages::dashboard::DashboardScreen;
use crate::ramp::session::{Session};
use crate::ramp::core::{new_project};

use serde::{Serialize, Deserialize};

#[derive(Debug, Component)]
pub struct NewProjectScreen(Stack, Page);

impl OnEvent for NewProjectScreen {}

impl AppPage for NewProjectScreen {}

impl NewProjectScreen {
    pub fn new(ctx: &mut Context) -> Result<Self, String> {
        // if ctx.state().get_named_mut::<Session>("session").is_none(){
        //     //create the session state if it doesn't exist
        //     println!("creating session token");
        //     let mut session = match Session::new() {
        //         Ok(s) => s,
        //         Err(e) => {
        //             ctx.state().set_named("error".to_string(), e);
        //             ctx.trigger_event(NavigateEvent(0));
        //             Session::default()
        //         }
        //     };
        //     println!("blank session token: {:?}", session);
        //     println!("populating session token");
        //     match session.get_all_paths() { //update the session state from config file
        //         Ok(())=> {},
        //         Err(e) => {
        //             ctx.state().set_named("error".to_string(), e);
        //             ctx.trigger_event(NavigateEvent(0));
        //         }
        //     };
        //     println!("populated session token: {:?}", session);
        //     println!("saving session token");
        //     ctx.state().set_named("session".to_string(), session);
        // }
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
            "Name your Project",
            //Size
            TextSize::H2,
            //style
            TextStyle::Heading,
            //alignment
            Align::Center,
            None
        );

        let mut name_input = TextInput::new(
            ctx,
            None,
            Some("Give Your Project a Name"),
            Some("Project name..."),
            Some("This name will be applied across the app template"),
            None
        );

        // Combine icon, heading, and subtext into page content
        let content = Content::new(
            ctx,
            // Vertically center items
            Offset::Center,
            // All items must be boxed as Box<dyn Drawable>
            vec![Box::new(text), Box::new(name_input)]
        );

        let bumper = Bumper::home(
            ctx, 
            ("Create", |ctx: &mut Context| {
                let page = Box::new(DashboardScreen::new(ctx).unwrap());
                ctx.trigger_event(NavigationEvent::Push(Some(page)))
            }), None
        );

        Ok(Self(Stack::default(), Page::new(header, content, Some(bumper))))
    }
}





// impl NewProjectScreen {
//     pub fn new(ctx: &mut Context) -> Self {
//         let mut session = ctx.state().get_named_mut::<Session>("session").unwrap();
//         println!("Session token: {:?}", session);
//         let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)));
//         // let mut session:&mut Session = ctx.state().get_named_mut::<Session>("session").unwrap();

//         // Create a header for the page
//         let header = Header::stack(
//             ctx,
//             Some(back),
//             "Create New Project", 
//             None
//         );

//         let font_size = ctx.theme.fonts.size;

//         // // Create the main heading text
//         // let text = Text::new(
//         //     ctx,
//         //     "Give your project a name",
//         //     TextStyle::Heading,
//         //     font_size.h2,
//         //     Align::Center
//         // );

//         // Create name input.
//         let mut name_input = TextInput::new(
//             ctx,
//             None,
//             Some("Give Your Project a Name"),
//             "Project name...",
//             Some("This name will be applied across the app template"),
//             TextInput::NO_ICON,
//             false
//         );

//         //TODO trim the name, make sure it doesn't contain any invalid characters and make sure its not blank
//         let create_button = Button::primary(
//             ctx,
//             "Create",
//             //on_click
//             |ctx: &mut Context| {
//                 let state = ctx.state();
//                 let mut session = state.get_named_mut::<Session>("session").unwrap();
//                 match new_project(&mut session, name_input.value()){
//                     Ok(())=> {
//                         drop(state);
//                         ctx.state().set_named("session".to_string(), session);
//                         ctx.trigger_event(NavigateEvent(2))
//                     },
//                     Err(e) => {
//                         drop(state);
//                         ctx.state().set_named("error".to_string(), e);
//                         ctx.trigger_event(NavigateEvent(0));
//                     }
//                 }
//                 //update state

//             }
//         );

//         let bumper = Bumper::single_button(ctx, create_button);

//         // Combine icon, heading, and subtext into page content
//         let content = Content::new(
//             ctx,
//             // Vertically center items
//             Offset::Center,
//             // All items must be boxed as Box<dyn Drawable>
//             vec![Box::new(name_input)]
//         );

//         NewProjectScreen(Stack::default(), Page::new(Some(header), content, Some(bumper)))
//     }
// }