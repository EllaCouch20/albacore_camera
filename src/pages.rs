use pelican_ui::{Component, Context};
use pelican_ui::drawable::{Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{OnEvent, Event};

use image::RgbaImage;

// use crate::pages::CameraRoll;
use crate::events::OpenSettingsEvent;
use crate::camera::AlbacoreCamera;
use crate::components::{CameraShutterButton, CameraRollButton, SettingsBumper};

use pelican_ui_std::{
    Stack, Offset, Header,
    Page, AppPage, IconButton,
    Content, Bumper,
};

#[derive(Debug, Component)]
pub struct CameraHome(Stack, Page, #[skip] Vec<RgbaImage>);

impl AppPage for CameraHome {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            // 0 => Ok(Box::new(CameraRoll::new(ctx, self.3.clone()))),
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
        let content = Content::new(ctx, Offset::Start, vec![Box::new(camera)]);
        let flip = Some(IconButton::navigation(ctx, "flip_camera", Box::new(|ctx: &mut Context| {})));
        let flash = Some(IconButton::navigation(ctx, "flash", Box::new(|ctx: &mut Context| {})));
        let header = Header::stack(ctx, flip, "", flash);
        CameraHome(Stack::default(), Page::new(Some(header), content, Some(AlbacoreBumpers::default(ctx))), roll)
    }
}

impl OnEvent for CameraHome {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(OpenSettingsEvent(a)) = event.downcast_ref::<OpenSettingsEvent>() {
            match a {
                true => *self.1.bumper() = Some(AlbacoreBumpers::settings(ctx)),
                false => *self.1.bumper() = Some(AlbacoreBumpers::default(ctx)),
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

    pub fn settings(ctx: &mut Context) -> Bumper {
        let bumper = SettingsBumper::new(ctx);
        Bumper::new(ctx, vec![Box::new(bumper)])
    }
}