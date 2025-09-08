use pelican_ui::{resources, Component, Context};
use pelican_ui::drawable::{Align, Drawable, Component, Image, ShapeType};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{Event, OnEvent, TickEvent};
use pelican_ui::LayoutResources;
use pelican_ui::hardware::ImageSettings;
use image::RgbaImage;

use pelican_ui_std::{
    IconButton, ExpandableText,
    Stack, RoundedRectangle, 
    AppPage, ExpandableImage, 
    Size, Offset, Padding,
    Header, Column, NavigateEvent,
    Page, Content, Slider, Bumper,
    Text, TextStyle, AspectRatioImage, EncodedImage,
};

use crate::events::{SetCameraSetting, NewPhotoEvent};
use crate::events::{ResetSetting, OpenSettingsEvent, NewSettingSelectedEvent, TakePhotoEvent, SelectImageEvent, SettingsSelect, PhotoBurstEvent};
use crate::components::{AlbacoreCamera, CameraBumper, EditSettingsBumper, PhotoWrap, CameraRollButton};

#[derive(Debug, Component)]
pub struct CameraHome(Stack, Page, #[skip] Option<String>, #[skip] Vec<RgbaImage>, #[skip] bool);

impl AppPage for CameraHome {
    fn has_nav(&self) -> bool { true }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraRoll::new(ctx, self.3.clone()))),
            _ => Err(self),
        }
    }
}

impl CameraHome {
    pub fn new(ctx: &mut Context, camera: Option<AlbacoreCamera>, roll: Vec<RgbaImage>) -> Self {
        ctx.theme.layout.content_max = f32::MAX;
        ctx.theme.layout.content_padding = 0.0;
        ctx.theme.layout.bumper_max = f32::MAX;

        let color = ctx.theme.colors.background.primary;
        let mut camera = camera.unwrap_or(AlbacoreCamera::new(ctx));
        let settings = camera.camera().as_ref().unwrap().get_settings().unwrap().clone();
        let view = CameraView::new(camera, CameraBumper::new(ctx, 0));
        let text_size = ctx.theme.fonts.size.h5;
        let text = Text::new(ctx, "Brightness", TextStyle::Heading, text_size, Align::Center);
        let bumper = EditSettingsBumper::new(ctx, settings);
        let content = Content::new(ctx, Offset::Start, vec![Box::new(view)]);
        CameraHome(Stack::default(), Page::new(None, content, None), None, roll, false)
    }

    fn settings(&mut self) -> Option<ImageSettings> {
        if let Some(view) = &mut self.1.content().find::<CameraView>() {
            let camera = view.camera().as_mut().unwrap().camera().as_mut().unwrap();
            return Some(camera.get_settings().unwrap().clone());
        }
        None
    }

    fn settings_bumper(&mut self) -> Option<&mut EditSettingsBumper> {
        if let Some(view) = self.1.content().find::<CameraView>() {
            return view.bumper().find::<EditSettingsBumper>();
        }
        None
    }
}

impl OnEvent for CameraHome {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(PhotoBurstEvent(vec)) = event.downcast_ref::<PhotoBurstEvent>() {
            // self.exposure_stack(vec).map(|e| self.3.push(e));
            // let width = vec[0].width();
            // let height = vec[0].height();
            // AlbacoreCamera::exposure_stack(
            //     &vec.into_iter().map(|img| img.clone().into_raw()).collect::<Vec<Vec<u8>>>()
            // ).map(|p| {
            //     RgbaImage::from_raw(width, height, p).map(|img| self.3.push(img));
            // });        
        } else if let Some(NewPhotoEvent(p)) = event.downcast_ref::<NewPhotoEvent>() {
            self.3.push(p.clone());
        } else if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(i) = self.2.clone() {
                let settings = self.settings().unwrap();
                if let Some(crb) = self.settings_bumper() {
                    crb.set_slider_value(SettingsValue::get(settings, i.to_string()));
                    self.2 = None;
                }
            }
        } else if let Some(s) = event.downcast_ref::<OpenSettingsEvent>() {
            *self.1.content().find::<CameraView>().unwrap().bumper().items() = match s {
                OpenSettingsEvent::Open => {
                    let settings = self.settings().unwrap();
                    vec![Box::new(EditSettingsBumper::new(ctx, settings))]
                },
                OpenSettingsEvent::Close => {
                    vec![Box::new(CameraBumper::new(ctx, 0))]
                }
            };
        } else if event.downcast_ref::<ResetSetting>().is_some() {
            if let Some(view) = &mut self.1.content().find::<CameraView>() {
                if let Some(crb) = view.bumper().find::<EditSettingsBumper>() {
                    let id = crb.get_current_label().clone().to_lowercase().replace(' ', "_");
                    crb.set_slider_to_default(ctx, id.to_string());
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
                let settings = camera.get_settings().unwrap().clone();

                match setting {
                    SetCameraSetting::Brightness(p) => camera.set_brightness((((p/100.0)*200.0)-100.0) as i16),
                    SetCameraSetting::Contrast(p) => camera.set_contrast(((p/100.0)*2.0)-1.0),
                    SetCameraSetting::Saturation(p) => camera.set_saturation(((p/100.0)*2.0)-1.0),
                    SetCameraSetting::Gamma(p) => camera.set_gamma((0.1+(p/100.0)*(3.0-0.1))),
                    SetCameraSetting::Exposure(p) => camera.set_exposure(((p/100.0)*4.0)-2.0),
                    SetCameraSetting::Temperature(p) => camera.set_temperature(2000.0+(p/100.0)*8000.0),
                    SetCameraSetting::WhiteBalanceR(p) => camera.set_white_balance_r(0.5+(p/100.0)*1.5),
                    SetCameraSetting::WhiteBalanceG(p) => camera.set_white_balance_g(0.5+(p/100.0)*1.5),
                    SetCameraSetting::WhiteBalanceB(p) => camera.set_white_balance_b(0.5+(p/100.0)*1.5),
                    SetCameraSetting::ExposureIso(p) => {
                        let value = SettingsValue::get(settings.clone(), "exposure_duration".to_string());
                        camera.set_exposure_and_iso(*p, value)
                    },
                    SetCameraSetting::ExposureDur(p) => {
                        let value = SettingsValue::get(settings.clone(), "exposure_iso".to_string());
                        camera.set_exposure_and_iso(value, *p)
                    }
                    _ => Ok(())
                };
            }
        }
        true
    }
}

pub struct SettingsValue;
impl SettingsValue {
    pub fn default(i: String) -> f32 {
        match i.as_str() {
            "brightness" => 50.0,
            "saturation" => 50.0,
            // "exposure" => ((settings.exposure + 2.0)/4.0)*100.0,
            "contrast" => 50.0,
            "temperature" => 56.5,
            "white_balance_r" => 33.0,
            "white_balance_g" => 33.0,
            "white_balance_b" => 33.0,
            "exposure_iso" => 50.0, 
            "exposure_duration" => 50.0,
            _ => 0.0
        }
    }

    pub fn get(settings: ImageSettings, i: String) -> f32 {
        match i.as_str() {
            "brightness" => ((settings.brightness as f32 + 100.0)/200.0)*100.0,
            "saturation" => ((settings.saturation + 1.0)/2.0)*100.0,
            // "exposure" => ((settings.exposure + 2.0)/4.0)*100.0,
            "contrast" => ((settings.contrast + 1.0)/2.0)*100.0,
            "temperature" => ((settings.temperature - 2000.0)/8000.0)*100.0,
            "white_balance_r" => ((settings.white_balance_r - 0.5)/1.5)*100.0,
            "white_balance_g" => ((settings.white_balance_g - 0.5)/1.5)*100.0,
            "white_balance_b" => ((settings.white_balance_b - 0.5)/1.5)*100.0,
            "exposure_iso" => settings.exposure_iso, 
            "exposure_duration" => settings.exposure_duration,
            _ => 0.0
        }
    }

    // let duration_seconds = 1.0 / 60.0;
    // let target_iso: f32 = 200.0;

    pub fn event(i: String) -> Box<dyn FnMut(&mut Context, f32)> {
        match i.as_str() {
            "brightness" => Box::new(|ctx: &mut Context, p: f32| {
                println!("Brightness action: {}", p);
                ctx.trigger_event(SetCameraSetting::Brightness(p))
            }),
            "saturation" => Box::new(|ctx: &mut Context, p: f32| {
                println!("Saturation action: {}", p);
                ctx.trigger_event(SetCameraSetting::Saturation(p))
            }),
            // "gamma" => Box::new(|ctx: &mut Context, p: f32| {
            //     println!("Gamma action: {}", p);
            //     ctx.trigger_event(SetCameraSetting::Gamma(p))
            // }),
            "exposure" => Box::new(|ctx: &mut Context, p: f32| {
                println!("Exposure action: {}", p);
                ctx.trigger_event(SetCameraSetting::Exposure(p))
            }),
            "contrast" => Box::new(|ctx: &mut Context, p: f32| {
                println!("Contrast action: {}", p);
                ctx.trigger_event(SetCameraSetting::Contrast(p))
            }),
            "temperature" => Box::new(|ctx: &mut Context, p: f32| {
                println!("Temperature action: {}", p);
                ctx.trigger_event(SetCameraSetting::Temperature(p))
            }),
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
            "exposure_duration" => Box::new(|ctx: &mut Context, p: f32| {
                println!("Exposure dur action: {}", p);
                ctx.trigger_event(SetCameraSetting::ExposureDur(p))
            }),
            "exposure_iso" => Box::new(|ctx: &mut Context, p: f32| {
                println!("ISO action: {}", p);
                ctx.trigger_event(SetCameraSetting::ExposureIso(p))
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
pub struct CameraRoll(Stack, Page, #[skip] Option<RgbaImage>, #[skip] Vec<RgbaImage>);

impl AppPage for CameraRoll {
    fn has_nav(&self) -> bool { true }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraHome::new(ctx, None, self.3.clone()))),
            1 => Ok(Box::new(ViewPhoto::new(ctx, self.2.unwrap(), self.3.clone()))),
            _ => Err(self),
        }
    }
}

impl OnEvent for CameraRoll {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(SelectImageEvent(image)) = event.downcast_ref::<SelectImageEvent>() {
            self.2 = Some(image.clone()) //Some((i.clone(), *s))
        }
        true
    }
}

impl CameraRoll {
    pub fn new(ctx: &mut Context, roll: Vec<RgbaImage>) -> Self {
        ctx.theme.layout = LayoutResources::default();
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let header = Header::stack(ctx, Some(back), "Library", None);
        let text_size = ctx.theme.fonts.size.md;
        let wrap = PhotoWrap::new(ctx, roll.clone());

        let content = Content::new(ctx, Offset::Start, vec![Box::new(wrap)]);
        CameraRoll(Stack::default(), Page::new(Some(header), content, None), None, roll)
    }
}

#[derive(Debug, Component)]
pub struct ViewPhoto(Stack, Page, #[skip] Vec<RgbaImage>);
impl OnEvent for ViewPhoto {}

impl AppPage for ViewPhoto {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraRoll::new(ctx, self.2.clone()))),
            _ => Err(self),
        }
    }
}

impl ViewPhoto {
    pub fn new(ctx: &mut Context, image: RgbaImage, roll: Vec<RgbaImage>) -> Self {
        ctx.theme.layout.bumper_max = f32::MAX;
        ctx.theme.layout.content_max = f32::MAX;
        ctx.theme.layout.content_padding = 0.0;
        let exp_img = ExpandableImage::new(ctx.assets.add_image(image.clone()), Some((image.width() as f32, image.height() as f32)));
        let content = Content::new(ctx, Offset::Center, vec![Box::new(exp_img)]);

        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let share = IconButton::navigation(ctx, "share", move |ctx: &mut Context| {
            ctx.hardware.share_image(image.clone());
        });
        
        let header = Header::stack(ctx, Some(back), "View Photo", Some(share));
        ViewPhoto(Stack::default(), Page::new(Some(header), content, None), roll)
    }
}
