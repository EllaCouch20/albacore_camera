use pelican_ui::{resources, include_assets, Theme, Component, Context, Plugins, Plugin, maverick_start, start, Application, PelicanEngine, MaverickOS};
use pelican_ui::drawable::{Drawable, Component, Image};
use pelican_ui_std::{Row, IconButton, Brand, Stack, RoundedRectangle, Interface, AppPage, ExpandableImage, Size, Offset, Padding};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{Event, OnEvent, TickEvent};
use pelican_ui::runtime::{Services, ServiceList};
use pelican_ui::hardware::{Camera, CameraError};
use profiles::plugin::ProfilePlugin;
use profiles::service::{Name, ProfileService};
use profiles::components::AvatarContentProfiles;
use messages::plugin::MessagesPlugin;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;

use bitcoin::service::BDKService;
use messages::service::{RoomsService, Rooms};

use bitcoin::pages::*;
use bitcoin::components::IconButtonBitcoin;
use messages::pages::*;
use messages::components::IconButtonMessages;
use profiles::pages::*;
use profiles::components::IconButtonProfiles;

// mod bdk;
// use bdk::BDKPlugin;
mod msg;
// use msg::MSGPlugin;
// use ucp_rust::UCPPlugin;

pub struct MyApp;
impl Services for MyApp {
    fn services() -> ServiceList {ServiceList(BTreeMap::new())}
}

impl Plugins for MyApp {
    fn plugins(ctx: &mut Context) -> Vec<Box<dyn Plugin>> {
        vec![]
    }
}

impl Application for MyApp {
    async fn new(ctx: &mut Context) -> Box<dyn Drawable> { 
        ctx.assets.include_assets(include_assets!("./resources"));
        let mut theme = Theme::default(&mut ctx.assets);
        theme.colors.button.secondary_default.background = theme.colors.background.primary;
        theme.colors.button.secondary_hover.background = theme.colors.background.secondary;
        theme.brand.illustrations.insert(ctx, "test", "images/darkglass_logo.png");
        ctx.theme = theme;
        App::new(ctx) 
    }
}

start!(MyApp);

#[derive(Debug, Component)]
pub struct App(Stack, RoundedRectangle, DarkGlassCamera, ScreenContent);

impl App {
    pub fn new(ctx: &mut Context) -> Box<Self> {
        let color = ctx.theme.colors.background.primary;
        Box::new(App(
            Stack(Offset::Center,Offset::End,Size::fill(),Size::fill(),Padding::default()), 
            RoundedRectangle::new(0.0, 8.0, color), DarkGlassCamera::new(ctx), ScreenContent::new(ctx)
        ))
    }
}

impl OnEvent for App {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {}
        true
    }
}

#[derive(Debug, Component)]
pub struct ScreenContent(Row, CameraButton, CameraButton, CameraButton);
impl OnEvent for ScreenContent {}
impl ScreenContent {
    pub fn new(ctx: &mut Context) -> Self {
        let button = IconButton::secondary(ctx, "camera", Box::new(|ctx: &mut Context| {
            println!("Camera button clicked!");
        }));

        let images = IconButton::secondary(ctx, "photos", Box::new(|ctx: &mut Context| {
            println!("Photoreel button clicked!");
        }));


        ScreenContent(Row::new(24.0, Offset::End,Size::Fit,Padding::new(24.0)), CameraButton::new(Some(images), false), CameraButton::new(Some(button), true), CameraButton::new(None, false))
    }
}


#[derive(Debug, Component)]
pub struct DarkGlassCamera(
    Stack, 
    ExpandableImage, 
    #[skip] Option<Camera>,
);

impl DarkGlassCamera {
    pub fn new(ctx: &mut Context) -> Self {
        let camera = match Camera::new_custom() {
            Ok(cam) => Some(cam),
            Err(e) => {
                println!("Failed to create custom camera: {:?}", e);
                None
            }
        };

        let test = ctx.theme.brand.illustrations.get("test").unwrap();
        
        DarkGlassCamera(
            Stack(Offset::Center,Offset::Center,Size::fill(),Size::fill(),Padding::default()),
            ExpandableImage::new(test), camera, 
        )
    }
}

impl OnEvent for DarkGlassCamera {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            if let Some(ref mut camera) = self.2 {
                match camera.get_frame() {
                    Some(raw_frame) => {
                        println!("Received frame: {}x{}", raw_frame.width(), raw_frame.height());
                    
                        let image = ctx.assets.add_image(raw_frame);
                        self.1.image().image = image;
                    },
                    _ => {}
                }
            }
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct CameraButton(Stack, Option<IconButton>);
impl OnEvent for CameraButton {}

impl CameraButton {
    pub fn new(icon: Option<IconButton>, expand: bool) -> Self {
        let size = if !expand {Size::Static(48.0)} else {Size::fill()};
        CameraButton(
            Stack(Offset::Center, Offset::Center, Size::Static(48.0), size, Padding::default()),
            icon
        )
    }
}

// #[derive(Debug, Component)]
// pub struct CameraReel(Stack, Image);
// impl OnEvent for CameraReel {}

// impl CameraReel {
//     pub fn new(ctx: &mut Context, image: Option<resources::Image>) -> Self {
//         let image = match image {
//             Some(i) => i,
//             None => ctx.theme.brand.illustrations.get("test").unwrap()
//         };
        
//         CameraReel(
//             Stack(Offset::Center, Offset::Center, Size::Static(48.0), Size::Static(48.0), Padding::default()),
//             Brand::new(image, (48.0, 48.0))
//         )
//     }

//     pub fn set_image(&mut self, new: Image) {
//         self.1 = new;
//     }
// }
