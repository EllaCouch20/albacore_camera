use pelican_ui::{resources, Component, Context};
use pelican_ui::drawable::{Align, Drawable, Component, Image, ShapeType};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{Event, OnEvent, TickEvent};
use pelican_ui::theme::LayoutResources;
use pelican_ui::hardware::ImageSettings;

use pelican_ui_std::{
    IconButton, 
    Stack, RoundedRectangle, 
    AppPage, ExpandableImage, 
    Size, Offset, Padding,
    Header, Column, NavigateEvent,
    Page, Content, Slider, Bumper,
    Text, TextStyle, Brand
};

use crate::events::SetCameraSetting;
use crate::events::{OpenSettingsEvent, NewSettingSelectedEvent, TakePhotoEvent, SelectImageEvent, SettingsSelect};
use crate::components::{AlbacoreCamera, CameraBumper, EditSettingsBumper, PhotoWrap, CameraRollButton};

#[derive(Debug, Component)]
pub struct CameraHome(Stack, Page, #[skip] Option<String>);

impl AppPage for CameraHome {
    fn has_nav(&self) -> bool { true }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraRoll::new(ctx))),
            _ => Err(self),
        }
    }
}

impl CameraHome {
    pub fn new(ctx: &mut Context, camera: Option<AlbacoreCamera>) -> Self {
        ctx.theme.layout.content_max = f32::MAX;
        ctx.theme.layout.content_padding = 0.0;
        ctx.theme.layout.bumper_max = f32::MAX;

        let color = ctx.theme.colors.background.primary;
        let camera = camera.unwrap_or(AlbacoreCamera::new(ctx));
        let view = CameraView::new(camera, CameraBumper::new(ctx, 0));
        let text_size = ctx.theme.fonts.size.h5;
        let text = Text::new(ctx, "Brightness", TextStyle::Heading, text_size, Align::Center);
        let bumper = EditSettingsBumper::new(ctx);
        let content = Content::new(ctx, Offset::Start, vec![Box::new(view)]);
        CameraHome(Stack::default(), Page::new(None, content, None), None)
    }
}

impl OnEvent for CameraHome {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(i) = &self.2 {
                if let Some(view) = &mut self.1.content().find::<CameraView>() {
                    let camera = view.camera().as_mut().unwrap().camera().as_mut().unwrap();
                    let settings = camera.get_settings().unwrap().clone();
                    if let Some(crb) = view.bumper().find::<EditSettingsBumper>() {
                        crb.set_slider_value(SettingsValue::get(settings, i.to_string()));
                        self.2 = None;
                    }
                }
            }
        } else if let Some(s) = event.downcast_ref::<OpenSettingsEvent>() {
            match s {
                OpenSettingsEvent::Open => {
                    *self.1.content().find::<CameraView>().unwrap().bumper().items() = vec![Box::new(EditSettingsBumper::new(ctx))]
                },
                OpenSettingsEvent::Close => {
                    *self.1.content().find::<CameraView>().unwrap().bumper().items() = vec![Box::new(CameraBumper::new(ctx, 0))]
                }
            }
        } else if let Some(SettingsSelect(id)) = event.downcast_ref::<SettingsSelect>() {
            if let Some(view) = &mut self.1.content().find::<CameraView>() {
                let camera = view.camera().as_mut().unwrap().camera().as_mut().unwrap();
                let settings = camera.get_settings().unwrap().clone();
                let value = SettingsValue::get(settings.clone(), id.to_string());
            
                if let Some(crb) = view.bumper().find::<EditSettingsBumper>() {
                    crb.set_text(id.to_string());
                    crb.set_slider(settings, ctx, id.to_string());
                    self.2 = Some(id.to_string());
                }
            }
        } else if let Some(setting) = event.downcast_ref::<SetCameraSetting>() {
            if let Some(camera) = self.1.content().find::<CameraView>().as_mut().unwrap().camera().as_mut().unwrap().camera() {
                match setting {
                    SetCameraSetting::Brightness(p) => camera.set_brightness((((p/100.0)*200.0)-100.0) as i16),
                    // SetCameraSetting::Contrast(p) => camera.set_contrast(((p/100.0)*2.0)-1.0),
                    // SetCameraSetting::Saturation(p) => camera.set_saturation(((p/100.0)*2.0)-1.0),
                    // SetCameraSetting::Gamma(p) => camera.set_gamma((0.1+(p/100.0)*(3.0-0.1))),
                    // SetCameraSetting::Exposure(p) => camera.set_exposure(((p/100.0)*4.0)-2.0),
                    // SetCameraSetting::Temperature(p) => camera.set_temperature(2000.0+(p/100.0)*8000.0),
                    SetCameraSetting::WhiteBalanceR(p) => camera.set_white_balance_r(0.5+(p/100.0)*1.5),
                    SetCameraSetting::WhiteBalanceG(p) => camera.set_white_balance_g(0.5+(p/100.0)*1.5),
                    SetCameraSetting::WhiteBalanceB(p) => camera.set_white_balance_b(0.5+(p/100.0)*1.5),
                };
            }
        }
        true
    }
}

pub struct SettingsValue;
impl SettingsValue {
    pub fn get(settings: ImageSettings, i: String) -> f32 {
        match i.as_str() {
            "brightness" => ((settings.brightness as f32 + 100.0)/200.0)*100.0,
            // "saturation" => ((settings.saturation + 1.0)/2.0)*100.0,
            // "gamma" => ((settings.temperature - 2000.0)/8000.0)*100.0,
            // "exposure" => ((settings.exposure + 2.0)/4.0)*100.0,
            // "contrast" => ((settings.contrast + 1.0)/2.0)*100.0,
            // "temperature" => ((settings.temperature - 2000.0)/8000.0)*100.0,
            "white_balance_r" => ((settings.white_balance_r - 0.5)/1.5)*100.0,
            "white_balance_g" => ((settings.white_balance_g - 0.5)/1.5)*100.0,
            "white_balance_b" => ((settings.white_balance_b - 0.5)/1.5)*100.0,
            _ => 0.0
        }
    }

    pub fn event(i: String) -> Box<dyn FnMut(&mut Context, f32)> {
        match i.as_str() {
            "brightness" => Box::new(|ctx: &mut Context, p: f32| {
                println!("Brightness action: {}", p);
                ctx.trigger_event(SetCameraSetting::Brightness(p))
            }),
            // "saturation" => Box::new(|ctx: &mut Context, p: f32| {
            //     println!("Saturation action: {}", p);
            //     ctx.trigger_event(SetCameraSetting::Saturation(p))
            // }),
            // "gamma" => Box::new(|ctx: &mut Context, p: f32| {
            //     println!("Gamma action: {}", p);
            //     ctx.trigger_event(SetCameraSetting::Gamma(p))
            // }),
            // "exposure" => Box::new(|ctx: &mut Context, p: f32| {
            //     println!("Exposure action: {}", p);
            //     ctx.trigger_event(SetCameraSetting::Exposure(p))
            // }),
            // "contrast" => Box::new(|ctx: &mut Context, p: f32| {
            //     println!("Contrast action: {}", p);
            //     ctx.trigger_event(SetCameraSetting::Contrast(p))
            // }),
            // "temperature" => Box::new(|ctx: &mut Context, p: f32| {
            //     println!("Temperature action: {}", p);
            //     ctx.trigger_event(SetCameraSetting::Temperature(p))
            // }),
            "white_balance_r" => Box::new(|ctx: &mut Context, p: f32| {
                println!("WhiteBalanceR action: {}", p);
                ctx.trigger_event(SetCameraSetting::WhiteBalanceR(p))
            }),
            "white_balance_g" => Box::new(|ctx: &mut Context, p: f32| {
                println!("WhiteBalanceG action: {}", p);
                ctx.trigger_event(SetCameraSetting::WhiteBalanceG(p))
            }),
            "white_balance_b" => Box::new(|ctx: &mut Context, p: f32| {
                println!("WhiteBalanceB action: {}", p);
                ctx.trigger_event(SetCameraSetting::WhiteBalanceB(p))
            }),
            _ => Box::new(move |ctx: &mut Context, p: f32| {
                println!("Unknown event: {} with value: {}", i, p);
            }),
        }
    }
}

#[derive(Debug, Component)]
pub struct CameraView(Stack, Option<AlbacoreCamera>, Bumper);
impl OnEvent for CameraView {}
impl CameraView {
    pub fn new(camera: AlbacoreCamera, bumper: Bumper) -> Self {
        CameraView(Stack(Offset::Center, Offset::End, Size::Fit, Size::Fit, Padding::default()), Some(camera), bumper)
    }

    pub fn bumper(&mut self) -> &mut Bumper { &mut self.2 }
    pub fn camera(&mut self) -> &mut Option<AlbacoreCamera> {&mut self.1}
}

#[derive(Debug, Component)]
pub struct CameraRoll(Stack, Page, #[skip] Option<Image>);

impl AppPage for CameraRoll {
    fn has_nav(&self) -> bool { true }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraHome::new(ctx, None))),
            1 => Ok(Box::new(ViewPhoto::new(ctx, self.2.unwrap()))),
            _ => Err(self),
        }
    }
}

impl OnEvent for CameraRoll {
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(SelectImageEvent(image, s)) = event.downcast_ref::<SelectImageEvent>() {
            self.2 = Some(Image{shape: ShapeType::RoundedRectangle(0.0, *s, 8.0), image: image.image.clone(), color: None}) //Some((i.clone(), *s))
        }
        true
    }
}

impl CameraRoll {
    pub fn new(ctx: &mut Context) -> Self {
        ctx.theme.layout = LayoutResources::default();
        let photo_wrap = PhotoWrap::new(ctx);
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let header = Header::stack(ctx, Some(back), "Library", None);
        let content = Content::new(ctx, Offset::Start, vec![Box::new(photo_wrap)]);
        CameraRoll(Stack::default(), Page::new(Some(header), content, None), None)
    }
}

#[derive(Debug, Component)]
pub struct ViewPhoto(Stack, Page);
impl OnEvent for ViewPhoto {}

impl AppPage for ViewPhoto {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraRoll::new(ctx))),
            _ => Err(self),
        }
    }
}

impl ViewPhoto {
    pub fn new(ctx: &mut Context, image: Image) -> Self {
        ctx.theme.layout.content_max = f32::MAX;
        ctx.theme.layout.content_padding = 0.0;
        ctx.theme.layout.bumper_max = f32::MAX;

        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let header = Header::stack(ctx, Some(back), "View Photo", None);
        let size = image.image.size();
        let image = ExpandableImage::new(image.image, Some((size.0 as f32, size.1 as f32)));
        let content = Content::new(ctx, Offset::Center, vec![Box::new(image)]);
        ViewPhoto(Stack::default(), Page::new(Some(header), content, None))
    }
}
