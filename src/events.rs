use pelican_ui::events::Event;
use pelican_ui::Context;
use pelican_ui::hardware::{ExposureMode, FocusMode, WhiteBalanceMode};
use image::RgbaImage;

#[derive(Debug, Clone)]
pub struct TakePhotoEvent;

impl Event for TakePhotoEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct ResetSetting;

impl Event for ResetSetting {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct OpenSettingsEvent(pub bool);

impl Event for OpenSettingsEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct SettingsSelect(pub String);

impl Event for SettingsSelect {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct EnumeratorSelectEvent(pub String);

impl Event for EnumeratorSelectEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}


#[derive(Debug, Clone)]
pub enum SetSettingEvent {
    Brightness(f32),
    Saturation(f32),
    Hue(f32),
    Contrast(f32),
    NoiseReduction(f32),
    Sharpness(f32),
    ExposureMode(ExposureMode),
    CustomExposureTime(f32),
    CustomExposureISO(f32),
    FocusMode(FocusMode),
    CustomFocusDistance(f32),
    WhiteBalanceMode(WhiteBalanceMode),
    WhiteBalanceGainsRed(f32),
    WhiteBalanceGainsGreen(f32),
    WhiteBalanceGainsBlue(f32),
    ToggleFlashlight,
    ToggleExposureStacking
}

impl Event for SetSettingEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}


#[derive(Debug, Clone)]
pub struct SelectImageEvent(pub RgbaImage);

impl Event for SelectImageEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct NewPhotoEvent(pub RgbaImage);

impl Event for NewPhotoEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}