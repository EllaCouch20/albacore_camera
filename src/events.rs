use pelican_ui::events::Event;
use pelican_ui::{resources, Context};
use pelican_ui::drawable::Image;

#[derive(Debug, Clone)]
pub struct TakePhotoEvent;

impl Event for TakePhotoEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct SelectImageEvent(pub Image, pub (f32, f32));

impl Event for SelectImageEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}


#[derive(Debug, Clone)]
pub struct NewSettingSelectedEvent(pub String);

impl Event for NewSettingSelectedEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}


#[derive(Debug, Clone)]
pub enum OpenSettingsEvent {
    Open,
    Close
}

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
pub enum SetCameraSetting {
    Brightness(f32),
    Contrast(f32),
    Saturation(f32),
    Gamma(f32),
    Exposure(f32),
    Temperature(f32),
    WhiteBalanceR(f32),
    WhiteBalanceG(f32),
    WhiteBalanceB(f32),
}

impl Event for SetCameraSetting {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}
