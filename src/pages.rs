use pelican_ui::{Component, Context, LayoutResources};
use pelican_ui::drawable::{Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{OnEvent, Event};
use pelican_ui::hardware::{CameraSettings, ExposureMode};

use image::RgbaImage;

// use crate::pages::CameraRoll;
use crate::events::{ResetSetting, NewPhotoEvent, OpenSettingsEvent, SetSettingEvent, SettingsSelect, SelectImageEvent};
use crate::camera::AlbacoreCamera;
use crate::components::{CameraShutterButton, CameraRollButton, SettingsBumper, PhotoWrap};

use pelican_ui_std::{
    Stack, Offset, Header,
    Page, AppPage, IconButton,
    Content, Bumper,
    NavigateEvent, ExpandableImage
};

#[derive(Debug, Component)]
pub struct CameraHome(Stack, Page, #[skip] Vec<RgbaImage>, #[skip] String);

impl AppPage for CameraHome {
    fn has_nav(&self) -> bool { true }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => {
                let camera = self.1.content().remove::<AlbacoreCamera>();
                Ok(Box::new(PhotoLibrary::new(ctx, camera, self.2.clone())))
            },
            _ => Err(self),
        }
    }
}

impl CameraHome {
    pub fn new(ctx: &mut Context, camera: Option<AlbacoreCamera>, roll: Vec<RgbaImage>) -> Self {
        ctx.theme.layout.content_max = f32::MAX;
        ctx.theme.layout.content_padding = 0.0;
        ctx.theme.layout.bumper_max = f32::MAX;

        let camera = camera.unwrap_or(AlbacoreCamera::new(ctx));
        let content = Content::new(ctx, Offset::Start, vec![Box::new(camera)]);
        let flip = Some(IconButton::navigation(ctx, "flip_camera", Box::new(|_ctx: &mut Context| {})));
        let flash = Some(IconButton::navigation(ctx, "flash", Box::new(|_ctx: &mut Context| {})));
        let header = Header::stack(ctx, flip, "", flash);
        CameraHome(Stack::default(), Page::new(Some(header), content, Some(AlbacoreBumpers::default(ctx))), roll, String::new())
    }
}

impl OnEvent for CameraHome {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(OpenSettingsEvent(a)) = event.downcast_ref::<OpenSettingsEvent>() {
            let s = self.1.content().find::<AlbacoreCamera>().as_mut().unwrap().camera().as_mut().unwrap().settings().unwrap().lock().unwrap().clone();
            match a {
                true => *self.1.bumper() = Some(AlbacoreBumpers::settings(ctx, s)),
                false => *self.1.bumper() = Some(AlbacoreBumpers::default(ctx)),
            }
        } else if event.downcast_ref::<ResetSetting>().is_some() {
            if let Some(settings_arc) = self.1.content().find::<AlbacoreCamera>().as_mut().unwrap().camera().as_mut().unwrap().settings().as_mut() {
                let mut settings_guard = settings_arc.lock().unwrap();
                let settings: &mut CameraSettings = &mut settings_guard;
                let default = CameraSettings::default();

                println!("RESETING {:?}", self.3);

                match self.3.as_str() {
                    "brightness" => settings.brightness = default.brightness,
                    "exposure" | "exposure_custom" => {
                        settings.exposure_mode = default.exposure_mode;
                        settings.custom_exposure = default.custom_exposure;
                    },
                    _ => {}
                }
                ctx.trigger_event(SettingsSelect(self.3.to_string()));
            }
        } else if let Some(NewPhotoEvent(p)) = event.downcast_ref::<NewPhotoEvent>() {
            self.2.push(p.clone());
        } else if let Some(SettingsSelect(setting)) = event.downcast_ref::<SettingsSelect>() {
            self.3 = setting.to_string();
            let s = self.1.content().find::<AlbacoreCamera>().as_mut().unwrap().camera().as_mut().unwrap().settings().unwrap().lock().unwrap().clone();
            let bumper_ref = &mut self.1.bumper().as_mut().unwrap().find::<SettingsBumper>();
            let bumper = bumper_ref.as_mut().unwrap();

            match setting.as_str() {
                "brightness" => {
                    let closure = move |ctx: &mut Context, p: f32| ctx.trigger_event(SetSettingEvent::Brightness(p));
                    bumper.set_first_slider(ctx, Some((Box::new(closure), s.brightness.unwrap_or(0.0), "Brightness")));
                    bumper.set_second_slider(ctx, None);
                    bumper.set_enumerator(ctx, None);
                },
                "exposure" => {
                    let items: Vec<(&str, Box<dyn FnMut(&mut Context)>)> = vec![
                        ("Auto", Box::new(|ctx: &mut Context| {
                            ctx.trigger_event(SetSettingEvent::ExposureMode(ExposureMode::Auto));
                            ctx.trigger_event(SettingsSelect("exposure".to_string()));
                        })),
                        ("Continuous", Box::new(|ctx: &mut Context| {
                            ctx.trigger_event(SetSettingEvent::ExposureMode(ExposureMode::Continuous));
                            ctx.trigger_event(SettingsSelect("exposure".to_string()));
                        })),
                        ("Custom", Box::new(|ctx: &mut Context| {
                            ctx.trigger_event(SetSettingEvent::ExposureMode(ExposureMode::Custom));
                            ctx.trigger_event(SettingsSelect("exposure_custom".to_string()))
                        })),
                    ];

                    match s.exposure_mode {
                        ExposureMode::Auto => {
                            bumper.set_enumerator(ctx, Some((items, "Exposure Mode", 0)));
                            bumper.set_first_slider(ctx, None);
                            bumper.set_second_slider(ctx, None);
                        },
                        ExposureMode::Continuous => {
                            bumper.set_enumerator(ctx, Some((items, "Exposure Mode", 1)));
                            bumper.set_first_slider(ctx, None);
                            bumper.set_second_slider(ctx, None);
                        },
                        ExposureMode::Custom => {
                            let duration = |ctx: &mut Context, p: f32| ctx.trigger_event(SetSettingEvent::CustomExposureTime(p));
                            let iso = |ctx: &mut Context, p: f32| ctx.trigger_event(SetSettingEvent::CustomExposureISO(p));
                            let values = s.custom_exposure.unwrap_or_default();

                            bumper.set_enumerator(ctx, Some((items, "Exposure Mode", 2)));
                            bumper.set_first_slider(ctx, Some((Box::new(duration), values.duration, "Duration")));
                            bumper.set_second_slider(ctx, Some((Box::new(iso), values.iso, "ISO")));
                        },
                    }
                },
                "exposure_custom" => {
                    if s.exposure_mode == ExposureMode::Custom {
                        let duration = move |ctx: &mut Context, p: f32| ctx.trigger_event(SetSettingEvent::CustomExposureTime(p));
                        let iso = move |ctx: &mut Context, p: f32| ctx.trigger_event(SetSettingEvent::CustomExposureISO(p));
                        let values = s.custom_exposure.unwrap_or_default();

                        bumper.set_first_slider(ctx, Some((Box::new(duration), values.duration, "Duration")));
                        bumper.set_second_slider(ctx, Some((Box::new(iso), values.iso, "ISO")));
                    } else {
                        ctx.trigger_event(SettingsSelect("exposure".to_string()))
                    }
                }
                _ => {}
            }
        }

        true
    }
}

pub struct AlbacoreBumpers;
impl AlbacoreBumpers {
    pub fn default(ctx: &mut Context) -> Bumper {
        let camera_roll = CameraRollButton::new(ctx);
        let shutter_button = CameraShutterButton::new(ctx);
        let settings = IconButton::secondary(ctx, "settings", Box::new(|ctx: &mut Context| ctx.trigger_event(OpenSettingsEvent(true))));
        Bumper::new(ctx, vec![Box::new(camera_roll), Box::new(shutter_button), Box::new(settings)])
    }

    pub fn settings(ctx: &mut Context, s: CameraSettings) -> Bumper {
        let bumper = SettingsBumper::new(ctx, Box::new(|ctx: &mut Context, p: f32| ctx.trigger_event(SetSettingEvent::Brightness(p))), s.brightness.unwrap_or(0.0), "Brightness");
        Bumper::new(ctx, vec![Box::new(bumper)])
    }
}


#[derive(Debug, Component)]
pub struct PhotoLibrary(Stack, Page, #[skip] Vec<RgbaImage>, #[skip] Option<AlbacoreCamera>,  #[skip] Option<RgbaImage>);

impl AppPage for PhotoLibrary {
    fn has_nav(&self) -> bool { false }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraHome::new(ctx, self.3.take(), self.2))),
            1 => Ok(Box::new(ViewPhoto::new(ctx, self.3.take(), self.2, self.4.unwrap().clone()))),
            _ => Err(self),
        }
    }
}

impl PhotoLibrary {
    pub fn new(ctx: &mut Context, camera: Option<AlbacoreCamera>, roll: Vec<RgbaImage>) -> Self {
        println!("Roll LENGTH {}", roll.len());
        ctx.theme.layout = LayoutResources::default();

        let photos = PhotoWrap::new(ctx, roll.clone());
        let content = Content::new(ctx, Offset::Start, vec![Box::new(photos)]);
        let back = IconButton::navigation(ctx, "left", Box::new(|ctx: &mut Context| {ctx.trigger_event(NavigateEvent(0))}));
        let header = Header::stack(ctx, Some(back), "Library", None);
        PhotoLibrary(Stack::default(), Page::new(Some(header), content, None), roll, camera, None)
    }
}

impl OnEvent for PhotoLibrary {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(SelectImageEvent(image)) = event.downcast_ref::<SelectImageEvent>() {
            self.4 = Some(image.clone()) //Some((i.clone(), *s))
        }
        true
    }
}


#[derive(Debug, Component)]
pub struct ViewPhoto(Stack, Page, #[skip] Vec<RgbaImage>, #[skip] Option<AlbacoreCamera>);
impl OnEvent for ViewPhoto {}

impl AppPage for ViewPhoto {
    fn has_nav(&self) -> bool { true }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(PhotoLibrary::new(ctx, self.3.take(), self.2))),
            _ => Err(self),
        }
    }
}

impl ViewPhoto {
    pub fn new(ctx: &mut Context, camera: Option<AlbacoreCamera>, roll: Vec<RgbaImage>, image: RgbaImage) -> Self {
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
        ViewPhoto(Stack::default(), Page::new(Some(header), content, None), roll, camera)
    }
}