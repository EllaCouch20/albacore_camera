use pelican_ui::{resources, Component, Context};
use pelican_ui::drawable::{Align, ShapeType, Drawable, Component, Image};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{Event, OnEvent, TickEvent, MouseEvent, MouseState};
use pelican_ui::hardware::Camera;
use pelican_ui::hardware::ImageSettings;

use image::RgbaImage;
use std::time::Instant;

// use crate::pages::CameraRoll;
use crate::service::LensRequest;
use crate::events::{TakePhotoEvent, SetCameraSetting, OpenSettingsEvent, NewSettingSelectedEvent, SelectImageEvent, SettingsSelect};
use crate::LensPlugin;
use crate::MyCameraRoll;
use crate::pages::SettingsValue;

use pelican_ui_std::{
    Row, IconButton, Text,
    Stack, ExpandableImage, 
    Size, Offset, Padding, 
    Wrap, TextStyle, Header,
    NavigateEvent, ExpandableText, 
    EncodedImage, AppPage, Column,
    Bumper, Icon, Bin, ButtonState,
    RoundedRectangle, ButtonStyle,
    ButtonWidth, ButtonSize, Button,
    Slider, Rectangle, Scroll,
    AdjustScrollEvent, ScrollAnchor,
    ElementID, NavigationButton,
    NavigatorSelect, Brand
};

pub struct CameraBumper;
impl CameraBumper {
    pub fn new(ctx: &mut Context, library_location: usize) -> Bumper {
        let settings = IconButton::ghost(ctx, "sliders", Box::new(|ctx: &mut Context| ctx.trigger_event(OpenSettingsEvent::Open)));

        let camera_roll = CameraRollButton::new(ctx, library_location);
        let shutter_button = ShutterButton::new(ctx);

        Bumper::new(ctx, vec![Box::new(camera_roll), Box::new(shutter_button), Box::new(settings)])
    }
}

#[derive(Debug, Component)]
pub struct ShutterButton(Stack, Image);

impl ShutterButton {
    pub fn new(ctx: &mut Context) -> Self {
        let color = ctx.theme.colors.text.heading;
        ShutterButton(
            Stack(Offset::Center, Offset::Center, Size::fill(), Size::Fit, Padding::default()),
            Icon::new(ctx, "camera_shutter", color, 64.0)
        )
    }
}

impl OnEvent for ShutterButton {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(MouseEvent { state: MouseState::Pressed, position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
            ctx.hardware.haptic();
            ctx.trigger_event(TakePhotoEvent);
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct CameraRollButton(Stack, Image, Bin<Stack, RoundedRectangle>, #[skip] usize, #[skip] u8, #[skip] u8);

impl CameraRollButton {
    pub fn new(ctx: &mut Context, i: usize) -> Self {
        let color = ctx.theme.colors.text.heading;
        let photos = ctx.state().get_or_default::<MyCameraRoll>().0.clone();
        let blank = ctx.theme.brand.illustrations.get("blank").unwrap();
        let image = photos.last().map(|(p, _)| EncodedImage::decode(ctx, &p)).unwrap_or(blank);
        let image = Image{shape: ShapeType::RoundedRectangle(0.0, (48.0, 48.0), 8.0), image, color: None};
        let layout = Stack(Offset::Center, Offset::Center, Size::Static(48.0), Size::Static(48.0), Padding::default());
        let len = ctx.state().get_or_default::<MyCameraRoll>().0.len();
        CameraRollButton(Stack::default(), image, Bin(layout, RoundedRectangle::new(1.0, 8.0, color)), i, len as u8, 0)
    }

    pub fn update(&mut self, ctx: &mut Context) {
        while self.5 > self.4 { 
            if let Some(p) = ctx.state().get_or_default::<MyCameraRoll>().0.clone().last() {
                let image = p.0.clone();
                self.1.image = EncodedImage::decode(ctx, &image);
            }
            self.4 += 1;
        }
    }
}

impl OnEvent for CameraRollButton {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(MouseEvent { state: MouseState::Pressed, position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
            ctx.trigger_event(NavigateEvent(self.3))
        } else if event.downcast_ref::<TickEvent>().is_some() {
            self.update(ctx);
        } else if event.downcast_ref::<TakePhotoEvent>().is_some() {
            self.5 += 1;
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct EditSettingsBumper(Column, Text, SettingsOptions, EditSlider);
impl OnEvent for EditSettingsBumper {}

impl EditSettingsBumper {
    pub fn new(ctx: &mut Context, settings: ImageSettings) -> Self {
        let text_size = ctx.theme.fonts.size.h5;
        let text = Text::new(ctx, "Brightness", TextStyle::Heading, text_size, Align::Center);
        let options = SettingsOptions::new(ctx);
        let action = SettingsValue::event("brightness".to_string());
        let value = SettingsValue::get(settings, "brightness".to_string());
        let edit_slider = EditSlider::new(ctx, value, action);
        let layout = Column::new(24.0, Offset::Center, Size::Fit, Padding::default());
        EditSettingsBumper(layout, text, options, edit_slider)
    }

    pub fn set_slider_value(&mut self, val: f32) {
        self.3.set_value(val)
    }

    pub fn set_text(&mut self, text: String) {
        self.1.text().spans[0].text = text.replace('_', " ").split_whitespace().map(|w| w[..1].to_uppercase() + &w[1..]).collect::<Vec<_>>().join(" ");
    }

    pub fn set_slider(&mut self, settings: ImageSettings, ctx: &mut Context, i: String) {
        let action = SettingsValue::event(i.to_string());
        *self.3.slider() = Slider::new(ctx, 50.0, None, None, action);
    }
}

#[derive(Debug, Component)]
pub struct SettingsOptions(Scroll, SettingsOptionsContent);

impl SettingsOptions {
    pub fn new(ctx: &mut Context) -> Self {
        let width = Size::custom(move |widths| (0.0, f32::MAX));
        let height = Size::custom(move |heights| (heights[0].0.min(48.0), 48.0));
        let layout = Scroll::horizontal(Offset::Start, Offset::Start, width, height, Padding::default(), ScrollAnchor::Start);
        SettingsOptions(layout, SettingsOptionsContent::new(ctx)) 
    }
}

impl OnEvent for SettingsOptions {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(AdjustScrollEvent::Horizontal(a)) = event.downcast_ref::<AdjustScrollEvent>() {
            self.0.adjust_scroll(*a);
        } else if let Some(MouseEvent { state: MouseState::Scroll(x, y), position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
            self.0.adjust_scroll(*x);
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct SettingsOptionsContent(Row, Vec<SettingsButton>);

impl SettingsOptionsContent {
    pub fn new(ctx: &mut Context) -> Self {
        let icons = vec!["brightness", "white_balance_r", "white_balance_g", "white_balance_b"];
        let children = icons.into_iter().enumerate().map(|(idx, icon)| {
            let closure = move |ctx: &mut Context| {
                ctx.trigger_event(SettingsSelect(icon.to_string()));
            };
            let button = IconButtonPreset::new(ctx, icon, 0 == idx, closure);
            SettingsButton::new(icon.to_string(), button)
        }).collect::<Vec<_>>();
        SettingsOptionsContent(Row::center(24.0), children)
    }
}

impl OnEvent for SettingsOptionsContent {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(SettingsSelect(id)) = event.downcast_ref::<SettingsSelect>() {
            self.1.iter_mut().for_each(|button| {
                let status = if button.id() == *id {ButtonState::Selected} else {ButtonState::Default};
                *button.inner().status() = status;
                button.inner().color(ctx, status);
            });
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct EditSlider(Row, Button, Slider);
impl OnEvent for EditSlider {}

impl EditSlider {
    pub fn new(ctx: &mut Context, start: f32, on_click: Box<dyn FnMut(&mut Context, f32)>) -> Self {
        let button = DoneButton::new(ctx, |ctx: &mut Context| ctx.trigger_event(OpenSettingsEvent::Close));
        let slider = Slider::new(ctx, start, None, None, on_click);
        EditSlider(Row::center(24.0), button, slider)
    }

    pub fn set_value(&mut self, val: f32) {self.2.set_value(val)}
    pub fn slider(&mut self) -> &mut Slider {&mut self.2}
}

#[derive(Debug, Component)]
pub struct PhotoWrap(Box<dyn Layout>, Vec<ImageButton>, Option<ExpandableText>);
impl OnEvent for PhotoWrap {}

impl PhotoWrap {
    pub fn new(ctx: &mut Context) -> Self {
        let text_size = ctx.theme.fonts.size.md;
        let my_images: Vec<(String, (f32, f32))> = ctx.state().get_or_default::<MyCameraRoll>().0.clone();
        let help_text = my_images.is_empty().then_some(ExpandableText::new(
            ctx, "Your camera roll is empty.\nTake a photo to get started.", 
            TextStyle::Primary, text_size, Align::Center, None
        ));

        let layout = match my_images.is_empty() {
            true => Box::new(Stack::center()) as Box<dyn Layout>,
            false => Box::new(Wrap::new(8.0, 8.0)) as Box<dyn Layout>
        };

        let my_photos = my_images.into_iter().map(|(i, s)| 
            ImageButton::new(ctx, i, s)
        ).collect();

        PhotoWrap(layout, my_photos, help_text)
    }
}

#[derive(Debug, Component)]
pub struct ImageButton(Stack, ExpandableImage, #[skip] String, #[skip] (f32, f32));
impl OnEvent for ImageButton {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(MouseEvent{state: MouseState::Pressed, position: Some(_)}) = event.downcast_ref::<MouseEvent>() {
            ctx.hardware.haptic();
            ctx.trigger_event(SelectImageEvent(self.2.to_string(), self.3));
            ctx.trigger_event(NavigateEvent(1));
        }
        true
    }
}

impl ImageButton {
    pub fn new(ctx: &mut Context, i: String, size: (f32, f32)) -> Self {
        let image = EncodedImage::decode(ctx, &i.clone());
        ImageButton(
            Stack(Offset::Center, Offset::Center, Size::Static(64.0), Size::Static(64.0), Padding::default()), 
            ExpandableImage::new(image, None), i.to_string(), size
        )
    }
}

#[derive(Debug, Component)]
pub struct AlbacoreCamera(Stack, ExpandableImage, #[skip] Option<Camera>, #[skip] Option<RgbaImage>);

impl AlbacoreCamera {
    pub fn new(ctx: &mut Context) -> Self {
        let blank = ctx.theme.brand.illustrations.get("blank").unwrap();
        
        AlbacoreCamera(
            Stack(Offset::Center,Offset::Center,Size::fill(),Size::fill(),Padding::default()),
            ExpandableImage::new(blank, None), Camera::new_custom().ok(), None
        )
    }

    pub fn camera(&mut self) -> &mut Option<Camera> {&mut self.2}
}

impl OnEvent for AlbacoreCamera {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            if let Some(ref mut camera) = self.2 {
                if let Some(raw_frame) = camera.get_frame() {
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
                let size = self.1.image().image.size();
                plugin.request(LensRequest::SavePhoto(image, (size.0 as f32, size.1 as f32)));
            }
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct SettingsButton(Stack, IconButton, #[skip] String);
impl OnEvent for SettingsButton {}

impl SettingsButton {
    pub fn new(id: String, button: IconButton) -> Self {
        SettingsButton(Stack::default(), button, id)
    }

    pub fn id(&self) -> String {
        self.2.clone()
    }

    pub fn inner(&mut self) -> &mut IconButton {
        &mut self.1
    }
}

struct DoneButton;
impl DoneButton {
    pub fn new(ctx: &mut Context, on_click: impl FnMut(&mut Context) + 'static) -> Button {
        Button::new(
            ctx,
            None,
            None,
            Some("Done"),
            None,
            ButtonSize::Medium,
            ButtonWidth::Hug,
            ButtonStyle::Primary,
            ButtonState::Default,
            Offset::Center,
            on_click,
            None,
        )
    }
}

struct IconButtonPreset;
impl IconButtonPreset {
    pub fn new(
        ctx: &mut Context, 
        icon: &'static str, 
        selected: bool,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> IconButton {
        let color = ctx.theme.colors.brand.primary;
        let state = if selected {ButtonState::Selected} else {ButtonState::Default};
        IconButton::new(
            ctx,
            icon,
            ButtonSize::Large,
            ButtonStyle::Ghost,
            state,
            Box::new(on_click),
            None,
        )
    }
}

