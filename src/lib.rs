use pelican_ui::{resources, include_assets, Theme, Component, Context, Plugins, Plugin, maverick_start, start, Application, PelicanEngine, MaverickOS};
use pelican_ui::drawable::{Align, ShapeType, Drawable, Component, Image};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{Event, OnEvent, TickEvent, MouseEvent, MouseState};
use pelican_ui::runtime::{Services, ServiceList};
use pelican_ui::hardware::{ApplicationSupport, Camera, ImageOrientation};
use profiles::plugin::ProfilePlugin;
use profiles::service::{ProfileService};
use image::RgbaImage;
use std::collections::BTreeMap;

use pelican_ui_std::{
    Row, IconButton, 
    Stack, RoundedRectangle, 
    Interface, AppPage, 
    ExpandableImage, Size, 
    Offset, Padding, Wrap,
    TextStyle, Header,
    Column, NavigateEvent,
    ExpandableText, EncodedImage
};


use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::io::BufReader;
use tempfile::NamedTempFile;

// mod bdk;
// use bdk::BDKPlugin;
// mod msg;
// use msg::MSGPlugin;
// use ucp_rust::UCPPlugin;

mod plugin;
use plugin::LensPlugin;
mod service;
use service::LensService;
use service::LensRequest;
use service::MyPhotos;

pub struct MyApp;
impl Services for MyApp {
    fn services() -> ServiceList {
        let mut services = ServiceList::default();
        services.insert::<ProfileService>();
        services.insert::<LensService>();
        services
        // ServiceList(BTreeMap::new())
    }
}

impl Plugins for MyApp {
    fn plugins(ctx: &mut Context) -> Vec<Box<dyn Plugin>> {
        vec![Box::new(ProfilePlugin::new(ctx)), Box::new(LensPlugin::new(ctx))]
        // vec![]
    }
}

impl Application for MyApp {
    async fn new(ctx: &mut Context) -> Box<dyn Drawable> { 
        ctx.assets.include_assets(include_assets!("./resources"));
        let mut theme = Theme::default(&mut ctx.assets);
        theme.colors.button.secondary_default.background = theme.colors.background.primary;
        theme.colors.button.secondary_hover.background = theme.colors.outline.secondary;
        theme.brand.illustrations.insert(ctx, "test", "images/darkglass_logo.png");
        ctx.theme = theme;
        App::new(ctx) 
    }
}

start!(MyApp);

#[derive(Debug, Component)]
pub struct App(Stack, Interface);

impl App {
    pub fn new(ctx: &mut Context) -> Box<Self> {
        let storage_path = ApplicationSupport::get().unwrap();
        std::fs::create_dir_all(&storage_path).unwrap();
        let path = storage_path.join("photos.json");
        let mut photos = Self::load_photos(&path);
 
        ctx.state().set(MyPhotos(photos));

        let home = CameraHome::new(ctx);
        let interface = Interface::new(ctx, Box::new(home), None);
        Box::new(App(Stack::default(), interface))
    }

    pub fn save_photos<P: AsRef<Path>>(path: P, photos: &Vec<String>) {
        let path = path.as_ref();
        let bytes = serde_json::to_vec_pretty(photos).expect("Could not vec to pretty");

        let mut tmp = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new("."))).expect("Could not write temp");
        tmp.write_all(&bytes).expect("Could not write all");
        tmp.flush().expect("Could not flush");
        tmp.persist(path).expect("Colud not persist");
    }

    pub fn load_photos<P: AsRef<Path>>(path: P) -> Vec<String> {
        let path = path.as_ref();
        if !path.exists() {
            return Vec::new();
        }
        let file = File::open(path).expect("Could not open path");
        let reader = BufReader::new(file);
        let contracts = serde_json::from_reader(reader).expect("Could not read from reader");
        contracts
    }
}

impl OnEvent for App {}

#[derive(Debug, Component)]
pub struct CameraHome(Stack, RoundedRectangle, DarkGlassCamera, ScreenContent);

impl AppPage for CameraHome {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraRoll::new(ctx))),
            // 1 => Ok(Box::new(Receive::new(ctx))),
            _ => Err(self),
        }
        // Ok(self)
    }
}

impl CameraHome {
    pub fn new(ctx: &mut Context) -> Self {
        let color = ctx.theme.colors.background.primary;
        CameraHome(
            Stack(Offset::Center,Offset::End,Size::fill(),Size::fill(),Padding::default()), 
            RoundedRectangle::new(0.0, 8.0, color), DarkGlassCamera::new(ctx), ScreenContent::new(ctx)
        )
    }
}

impl OnEvent for CameraHome {
    // fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
    //     if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {}
    //     true
    // }
}

#[derive(Debug, Component)]
pub struct ScreenContent(Row, CameraButton, CameraButton, CameraButton);
impl OnEvent for ScreenContent {}
impl ScreenContent {
    pub fn new(ctx: &mut Context) -> Self {
        let button = IconButton::secondary(ctx, "camera", Box::new(|ctx: &mut Context| {
            ctx.trigger_event(TakePhotoEvent)
        }));

        let images = IconButton::secondary(ctx, "photos", Box::new(|ctx: &mut Context| {
            println!("Photoreel button clicked!");
            ctx.trigger_event(NavigateEvent(0));
        }));


        ScreenContent(Row::new(24.0, Offset::Center,Size::Fit,Padding::new(24.0)), CameraButton::new(Some(images), false), CameraButton::new(Some(button), true), CameraButton::new(None, false))
    }
}


#[derive(Debug, Component)]
pub struct DarkGlassCamera(
    Stack, 
    ExpandableImage, 
    #[skip] Option<Camera>,
    #[skip] Option<RgbaImage>,
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
            ExpandableImage::new(test), camera, None
        )
    }
}

impl OnEvent for DarkGlassCamera {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            if let Some(ref mut camera) = self.2 {
                if let Some(raw_frame) = camera.get_frame() {
                    // println!("Received frame: {}x{}", raw_frame.width(), raw_frame.height());
                    self.3 = Some(raw_frame.clone());
                    let image = ctx.assets.add_image(raw_frame);
                    self.1.image().image = image;
                }
            }
        } else if let Some(TakePhotoEvent) = event.downcast_ref::<TakePhotoEvent>() {
            if let Some(rgba) = &self.3 {
                let mut guard = ctx.get::<LensPlugin>();
                let plugin = guard.get().0;
                let image = EncodedImage::encode_rgba(rgba.clone());
                println!("GOT IMAGE. MAKING REQUEST");
                plugin.request(LensRequest::SavePhoto(image));
            }
        }
        true
    }
}


#[derive(Debug, Component)]
pub struct CameraRoll(Column, Header, PhotoWrap, #[skip] Option<resources::Image>);

impl AppPage for CameraRoll {
    fn has_nav(&self) -> bool { true }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraHome::new(ctx))),
            1 => Ok(Box::new(ViewPhoto::new(ctx, self.3.take().unwrap()))),
            _ => Err(self),
        }
        // Ok(self)
    }
}

impl CameraRoll {
    pub fn new(ctx: &mut Context) -> Self {
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        CameraRoll(
            Column::new(24.0, Offset::Center, Size::fill(), Padding::new(24.0)), 
            Header::stack(ctx, Some(back), "Camera Roll", None),
            PhotoWrap::new(ctx),
            None
        )
    }
}

impl OnEvent for CameraRoll {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(SelectImageEvent(i)) = event.downcast_ref::<SelectImageEvent>() {
            self.3 = Some(i.clone())
        }
        true
    }
}


#[derive(Debug, Component)]
pub struct ViewPhoto(Column, Header, ExpandableImage);

impl AppPage for ViewPhoto {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraRoll::new(ctx))),
            // 1 => Ok(Box::new(Receive::new(ctx))),
            _ => Err(self),
        }
        // Ok(self)
    }
}

impl ViewPhoto {
    pub fn new(ctx: &mut Context, image: resources::Image) -> Self {
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        ViewPhoto(
            Column::new(24.0, Offset::Center, Size::fill(), Padding::new(24.0)), 
            Header::stack(ctx, Some(back), "View Photo", None),
            ExpandableImage::new(image)
        )
    }
}

impl OnEvent for ViewPhoto {
    // fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
    //     if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {}
    //     true
    // }
}

#[derive(Debug, Component)]
pub struct PhotoWrap(Box<dyn Layout>, Vec<ImageButton>, Option<ExpandableText>);

impl PhotoWrap {
    pub fn new(ctx: &mut Context) -> Self {
        let text_size = ctx.theme.fonts.size.md;
        let my_images: Vec<String> = ctx.state().get_or_default::<MyPhotos>().0.clone();
        let help_text = (my_images.is_empty()).then_some(
            ExpandableText::new(ctx, "Your camera roll is empty.\nTake a photo to get started.", TextStyle::Primary, text_size, Align::Center, None)
        );
        let layout = if my_images.is_empty() {
            Box::new(Stack::center()) as Box<dyn Layout>
        } else {
            Box::new(Wrap::new(8.0, 8.0)) as Box<dyn Layout>
        };
        let my_photos = my_images.into_iter().map(|i| {
            let image = EncodedImage::decode(ctx, &i);
            ImageButton::new(image)
        }).collect();
        PhotoWrap(layout, my_photos, help_text)
    }
}

impl OnEvent for PhotoWrap {
    // fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
    //     if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
    //         if let Some(ref mut camera) = self.2 {
    //             match camera.get_frame() {
    //                 Some(raw_frame) => {
    //                     println!("Received frame: {}x{}", raw_frame.width(), raw_frame.height());
                    
    //                     let image = ctx.assets.add_image(raw_frame);
    //                     self.1.image().image = image;
    //                 },
    //                 _ => {}
    //             }
    //         }
    //     }
    //     true
    // }
}

#[derive(Debug, Component)]
pub struct CameraButton(Stack, Option<IconButton>);
impl OnEvent for CameraButton {}

impl CameraButton {
    pub fn new(icon: Option<IconButton>, expand: bool) -> Self {
        let size = if !expand {Size::Fit} else {Size::fill()};
        CameraButton(
            Stack(Offset::Center, Offset::Center, size, Size::Fit, Padding::default()),
            icon
        )
    }
}


#[derive(Debug, Component)]
pub struct ImageButton(Stack, Option<Image>);
impl OnEvent for ImageButton {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(MouseEvent{state: MouseState::Pressed, position: Some(_)}) = event.downcast_ref::<MouseEvent>() {
            ctx.hardware.haptic();
            ctx.trigger_event(SelectImageEvent(self.1.take().unwrap().image));
            ctx.trigger_event(NavigateEvent(1));
        }
        true
    }
}

impl ImageButton {
    pub fn new(image: resources::Image) -> Self {
        let image = Image{shape: ShapeType::RoundedRectangle(0.0, (64.0, 64.0), 8.0), image, color: None};
        ImageButton(Stack::default(), Some(image))
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

#[derive(Debug, Clone)]
pub struct TakePhotoEvent;

impl Event for TakePhotoEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct SelectImageEvent(pub resources::Image);

impl Event for SelectImageEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}