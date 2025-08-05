use pelican_ui::{resources, Component, Context};
use pelican_ui::drawable::{Align, ShapeType, Drawable, Component, Image};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{Event, OnEvent, TickEvent, MouseEvent, MouseState};
use pelican_ui::hardware::Camera;
use image::RgbaImage;

use crate::pages::CameraRoll;
use crate::service::LensRequest;
use crate::events::{TakePhotoEvent, SelectImageEvent, SetCameraSetting};
use crate::LensPlugin;
use crate::MyPhotos;

use pelican_ui_std::{
    Row, IconButton, 
    Stack, ExpandableImage, 
    Size, Offset, Padding, 
    Wrap, TextStyle, Header,
    NavigateEvent, ExpandableText, 
    EncodedImage, AppPage, Column,
};

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

        let settings = IconButton::secondary(ctx, "settings", Box::new(|ctx: &mut Context| {
            println!("Settings button clicked!");
            ctx.trigger_event(NavigateEvent(1));
        }));


        ScreenContent(Row::new(24.0, Offset::Center,Size::Fit,Padding::new(24.0)), CameraButton::new(Some(images), false), CameraButton::new(Some(button), true), CameraButton::new(Some(settings), false))
    }
}

#[derive(Debug, Component)]
pub struct PhotoWrap(Box<dyn Layout>, Vec<ImageButton>, Option<ExpandableText>);
impl OnEvent for PhotoWrap {}

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
            ExpandableImage::new(test, (1.0, 1.0)), camera, None
        )
    }

    pub fn camera(&mut self) -> &mut Option<Camera> {&mut self.2}
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



// self.brightness = self.brightness.clamp(-100, 100);
// self.contrast = self.contrast.clamp(-1.0, 1.0);
// self.saturation = self.saturation.clamp(-1.0, 1.0);
// self.gamma = self.gamma.clamp(0.1, 3.0);
// self.white_balance_r = self.white_balance_r.clamp(0.5, 2.0);
// self.white_balance_g = self.white_balance_g.clamp(0.5, 2.0);
// self.white_balance_b = self.white_balance_b.clamp(0.5, 2.0);
// self.exposure = self.exposure.clamp(-2.0, 2.0);
// self.temperature = self.temperature.clamp(2000.0, 10000.0);