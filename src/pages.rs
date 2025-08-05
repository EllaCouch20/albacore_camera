use pelican_ui::{resources, Component, Context};
use pelican_ui::drawable::{Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{Event, OnEvent, TickEvent};

use pelican_ui_std::{
    IconButton, 
    Stack, RoundedRectangle, 
    AppPage, ExpandableImage, 
    Size, Offset, Padding,
    Header, Column, NavigateEvent,
    Page, Content, Slider,
};

use crate::events::SetCameraSetting;
use crate::events::SelectImageEvent;
use crate::components::{PhotoWrap, ScreenContent, DarkGlassCamera};

#[derive(Debug, Component)]
pub struct CameraHome(Stack, RoundedRectangle, Option<DarkGlassCamera>, ScreenContent);
impl OnEvent for CameraHome {}
impl AppPage for CameraHome {
    fn has_nav(&self) -> bool { true }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraRoll::new(ctx))),
            1 => Ok(Box::new(CameraSettings::new(ctx, self.2.take().unwrap()))),
            _ => Err(self),
        }
        // Ok(self)
    }
}

impl CameraHome {
    pub fn new(ctx: &mut Context, camera: Option<DarkGlassCamera>) -> Self {
        let color = ctx.theme.colors.background.primary;
        let camera = match camera {
            Some(c) => c,
            None => DarkGlassCamera::new(ctx),
        };

        CameraHome(
            Stack(Offset::Center,Offset::End,Size::fill(),Size::fill(),Padding::default()), 
            RoundedRectangle::new(0.0, 8.0, color), Some(camera), ScreenContent::new(ctx),
        )
    }
}


#[derive(Debug, Component)]
pub struct CameraRoll(Column, Header, PhotoWrap, #[skip] Option<resources::Image>);

impl AppPage for CameraRoll {
    fn has_nav(&self) -> bool { true }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraHome::new(ctx, None))),
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
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(SelectImageEvent(i)) = event.downcast_ref::<SelectImageEvent>() {
            self.3 = Some(i.clone())
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct CameraSettings(Stack, Page, #[skip] Option<DarkGlassCamera>, #[skip] bool);

impl AppPage for CameraSettings {
    fn has_nav(&self) -> bool { true }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(CameraHome::new(ctx, self.2.take()))),
            // 1 => Ok(Box::new(Receive::new(ctx))),
            _ => Err(self),
        }
        // Ok(self)
    }
}

impl CameraSettings {
    pub fn new(ctx: &mut Context, mut camera: DarkGlassCamera) -> Self {
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let header = Header::stack(ctx, Some(back), "Camera Settings", None);

        let brightness = Slider::new(ctx, "Brightness", None, |ctx: &mut Context, p: f32| ctx.trigger_event(SetCameraSetting::Brightness(p)));
        let contrast = Slider::new(ctx, "Contrast", None, |ctx: &mut Context, p: f32| ctx.trigger_event(SetCameraSetting::Contrast(p)));
        let saturation = Slider::new(ctx, "Saturation", None, |ctx: &mut Context, p: f32| ctx.trigger_event(SetCameraSetting::Saturation(p)));
        let gamma = Slider::new(ctx, "Gamma", None, |ctx: &mut Context, p: f32| ctx.trigger_event(SetCameraSetting::Gamma(p)));
        let exposure = Slider::new(ctx, "Exposure", None, |ctx: &mut Context, p: f32| ctx.trigger_event(SetCameraSetting::Exposure(p)));
        let temperature = Slider::new(ctx, "Temperature", None, |ctx: &mut Context, p: f32| ctx.trigger_event(SetCameraSetting::Temperature(p)));
        let white_balance_r = Slider::new(ctx, "White Balance R", None, |ctx: &mut Context, p: f32| ctx.trigger_event(SetCameraSetting::WhiteBalanceR(p)));
        let white_balance_g = Slider::new(ctx, "White Balance G", None, |ctx: &mut Context, p: f32| ctx.trigger_event(SetCameraSetting::WhiteBalanceG(p)));
        let white_balance_b = Slider::new(ctx, "White Balance B", None, |ctx: &mut Context, p: f32| ctx.trigger_event(SetCameraSetting::WhiteBalanceB(p)));

        let content = Content::new(Offset::Start, vec![
            Box::new(brightness),
            Box::new(contrast),
            Box::new(saturation),
            Box::new(gamma),
            Box::new(exposure),
            Box::new(temperature),
            Box::new(white_balance_r),
            Box::new(white_balance_g),
            Box::new(white_balance_b),
        ]);

        CameraSettings(Stack::default(), Page::new(Some(header), content, None), Some(camera), false)
    }
}

impl OnEvent for CameraSettings {
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(tick) = event.downcast_ref::<TickEvent>() {
            if !self.3 {
                println!("Adjusting SLIDERS");
                self.3 = true;
                let settings = self.2.as_mut().unwrap().camera().as_ref().unwrap().get_settings().unwrap().clone();
                let content = &mut self.1.content();
                content.find_at::<Slider>(0).unwrap().set_value(((settings.brightness as f32 + 100.0)/200.0)*100.0);
                content.find_at::<Slider>(1).unwrap().set_value(((settings.contrast + 1.0)/2.0)*100.0);
                content.find_at::<Slider>(2).unwrap().set_value(((settings.saturation + 1.0)/2.0)*100.0);
                content.find_at::<Slider>(3).unwrap().set_value(((settings.gamma - 0.1)/2.9)*100.0);
                content.find_at::<Slider>(4).unwrap().set_value(((settings.exposure + 2.0)/4.0)*100.0);
                content.find_at::<Slider>(5).unwrap().set_value(((settings.temperature - 2000.0)/8000.0)*100.0);
                content.find_at::<Slider>(6).unwrap().set_value(((settings.white_balance_r - 0.5)/1.5)*100.0);
                content.find_at::<Slider>(7).unwrap().set_value(((settings.white_balance_g - 0.5)/1.5)*100.0);
                content.find_at::<Slider>(8).unwrap().set_value(((settings.white_balance_b - 0.5)/1.5)*100.0);
            }
        } else if let Some(setting) = event.downcast_ref::<SetCameraSetting>() {
            println!("SOME {:?}", setting);
            if let Some(camera) = self.2.as_mut().unwrap().camera() {
                println!(" Found Camera ");
                match setting {
                    SetCameraSetting::Brightness(p) => camera.update_settings(|settings| settings.brightness = (((p/100.0)*200.0)-100.0) as i16),
                    SetCameraSetting::Contrast(p) => camera.update_settings(|settings| settings.contrast = ((p/100.0)*2.0)-1.0),
                    SetCameraSetting::Saturation(p) => camera.update_settings(|settings| settings.saturation = ((p/100.0)*2.0)-1.0),
                    SetCameraSetting::Gamma(p) => camera.update_settings(|settings| settings.gamma = (0.1+(p/100.0)*(3.0-0.1))),
                    SetCameraSetting::Exposure(p) => camera.update_settings(|settings| settings.exposure = ((p/100.0)*4.0)-2.0),
                    SetCameraSetting::Temperature(p) => camera.update_settings(|settings| settings.temperature = 2000.0+(p/100.0)*8000.0),
                    SetCameraSetting::WhiteBalanceR(p) => camera.update_settings(|settings| settings.white_balance_r = 0.5+(p/100.0)*1.5),
                    SetCameraSetting::WhiteBalanceG(p) => camera.update_settings(|settings| settings.white_balance_g = 0.5+(p/100.0)*1.5),
                    SetCameraSetting::WhiteBalanceB(p) => camera.update_settings(|settings| settings.white_balance_b = 0.5+(p/100.0)*1.5),
                };
            }
        }
        true
    }
}

// // Update individual settings
// camera.update_settings(|settings| {
//     settings.brightness = 25;           // Range: -100 to 100
//     settings.contrast = 0.3;            // Range: -1.0 to 1.0
//     settings.saturation = 0.2;          // Range: -1.0 to 1.0
//     settings.gamma = 2.4;               // Range: 0.1 to 3.0
//     settings.exposure = 0.5;            // Range: -2.0 to 2.0
//     settings.temperature = 5500.0;      // Range: 2000.0 to 10000.0 (Kelvin)
//     settings.white_balance_r = 1.1;     // Range: 0.5 to 2.0
//     settings.white_balance_g = 1.0;     // Range: 0.5 to 2.0
//     settings.white_balance_b = 0.9;     // Range: 0.5 to 2.0
// });


#[derive(Debug, Component)]
pub struct ViewPhoto(Column, Header, ExpandableImage);
impl OnEvent for ViewPhoto {}

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
            ExpandableImage::new(image, (1.0, 1.0))
        )
    }
}
