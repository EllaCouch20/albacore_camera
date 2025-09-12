use pelican_ui::{Component, Context};
use pelican_ui::drawable::{Drawable, Component, Image, Align, ShapeType};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{Event, OnEvent, TickEvent, MouseEvent, MouseState};
use pelican_ui::hardware::Camera;

use image::RgbaImage;

// use crate::pages::CameraRoll;
use crate::service::LensRequest;
use crate::LensPlugin;
use crate::events::{TakePhotoEvent, SettingsSelect, OpenSettingsEvent};
use crate::layout::SpaceEvenly;
use crate::MyCameraRoll;

use pelican_ui_std::{
    Stack, ExpandableImage,
    EncodedImage, Padding,
    Bin, Rectangle, Size,
    Offset, AspectRatioImage,
    RoundedRectangle, Icon,
    ButtonState, IconButton,
    ButtonSize, ButtonStyle,
    AdjustScrollEvent, Row,
    ScrollAnchor, Scroll,
    Column, TextStyle, Text,
    ExpandableText, Slider,
    Button, ButtonWidth,
};

#[derive(Debug, Component)]
pub struct CameraRollButton(Stack, Image, RoundedRectangle, #[skip] usize);

impl CameraRollButton {
    pub fn new(ctx: &mut Context) -> Self {
        let photos = ctx.state().get::<MyCameraRoll>().unwrap().0.clone();
        let blank = ctx.theme.brand.illustrations.get("blank").unwrap();
        let image = photos.last().map(|(p, _)| EncodedImage::decode(ctx, p)).unwrap_or(blank);
        let image = Image{shape: ShapeType::RoundedRectangle(0.0, (48.0, 48.0), 8.0, 0.0), image, color: None};
        let outline = RoundedRectangle::new(1.0, 8.0, ctx.theme.colors.outline.primary);
        let layout = Stack(Offset::Center, Offset::Center, Size::Static(48.0), Size::Static(48.0), Padding::default());
        CameraRollButton(layout, image, outline, photos.len())
    }
}

impl OnEvent for CameraRollButton {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if event.downcast_ref::<TickEvent>().is_some() {
            let photos = ctx.state().get::<MyCameraRoll>().unwrap().0.clone();
            if photos.len() > self.3 {
                let blank = ctx.theme.brand.illustrations.get("blank").unwrap();
                let image = photos.last().map(|(p, _)| EncodedImage::decode(ctx, p)).unwrap_or(blank);
                let image = Image{shape: ShapeType::RoundedRectangle(0.0, (48.0, 48.0), 8.0, 0.0), image, color: None};
                self.3 = photos.len();
            }
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct CameraShutterButton(Stack, Image);

impl CameraShutterButton {
    pub fn new(ctx: &mut Context) -> Self {
        let color = ctx.theme.colors.text.heading;
        CameraShutterButton(
            Stack(Offset::Center, Offset::Center, Size::fill(), Size::Fit, Padding::default()),
            Icon::new(ctx, "camera_shutter", color, 64.0)
        )
    }
}

impl OnEvent for CameraShutterButton {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(MouseEvent { state: MouseState::Pressed, position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
            ctx.hardware.haptic();
            self.1 = Icon::new(ctx, "camera_shutter_active", ctx.theme.colors.text.heading, 64.0);
            ctx.trigger_event(TakePhotoEvent);
        } else if let Some(MouseEvent { state: MouseState::Released, position: _ }) = event.downcast_ref::<MouseEvent>() {
            self.1 = Icon::new(ctx, "camera_shutter", ctx.theme.colors.text.heading, 64.0);
        } else if let Some(MouseEvent { state: MouseState::Moved, position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
            self.1 = Icon::new(ctx, "camera_shutter_active", ctx.theme.colors.text.heading, 64.0);
        } else if let Some(MouseEvent { state: MouseState::Moved, position: None }) = event.downcast_ref::<MouseEvent>() {
            self.1 = Icon::new(ctx, "camera_shutter", ctx.theme.colors.text.heading, 64.0);
        }
        true
    }
}


#[derive(Debug, Component)]
pub struct SettingsBumper(Column, SettingsOptions, Bin<Stack, Rectangle>, SettingsDetails, Slider);
impl OnEvent for SettingsBumper {}

impl SettingsBumper {
    pub fn new(ctx: &mut Context) -> Self {
        let color = ctx.theme.colors.outline.secondary;
        let layout = Stack(Offset::Center, Offset::Center, Size::fill(), Size::Static(3.0), Padding::default());
        SettingsBumper(
            Column::new(16.0, Offset::Center, Size::fill(), Padding::default()),
            SettingsOptions::new(ctx),
            Bin(layout, Rectangle::new(color, 0.0)),
            SettingsDetails::new(ctx, "Brightness", "50%"),
            Slider::new(ctx, 0.0, None, None, |ctx: &mut Context, p: f32| {}),
        )
    }
}

#[derive(Debug, Component)]
pub struct SettingsOptions(Scroll, SettingsOptionsContent);

impl SettingsOptions {
    pub fn new(ctx: &mut Context) -> Self {
        let width = Size::custom(move |widths| (0.0, f32::MAX));
        let layout = Scroll::horizontal(Offset::Start, Offset::Start, width, Size::Fit, Padding::default(), ScrollAnchor::Start);
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
        let icons = vec![
            "gamma", "contrast",
            "brightness", "saturation", 
            "temperature", "white_balance_r", 
            "white_balance_g", "white_balance_b", 
            "exposure_iso", "exposure_duration", 
            "exposure_stacking"
        ];

        let mut icon_buttons = vec![SettingsButton::new("left".to_string(), IconButtonPreset::new(ctx, "left", false, |ctx: &mut Context| ctx.trigger_event(OpenSettingsEvent(false))))];
        icons.into_iter().enumerate().for_each(|(idx, icon)| {
            let closure = move |ctx: &mut Context| {ctx.trigger_event(SettingsSelect(icon.to_string()))};
            let button = IconButtonPreset::new(ctx, icon, 0 == idx, closure);
            icon_buttons.push(SettingsButton::new(icon.to_string(), button))
        });
        SettingsOptionsContent(Row::center(24.0), icon_buttons)
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
pub struct SettingsDetails(Row, Text, ExpandableText, Button);

impl OnEvent for SettingsDetails {}

impl SettingsDetails {
    pub fn new(ctx: &mut Context, first: &str, second: &str) -> Self {
        let font_size = ctx.theme.fonts.size;
        let first = Text::new(ctx, first, TextStyle::Heading, font_size.h5, Align::Left);
        let second = ExpandableText::new(ctx, second, TextStyle::Primary, font_size.h5, Align::Left, None);
        let button = Button::new(ctx, None, None, Some("Reset"), None,
            ButtonSize::Medium, ButtonWidth::Hug, ButtonStyle::Primary,
            ButtonState::Default, Offset::Center, |ctx: &mut Context| {}, None
        );
        SettingsDetails(Row::new(6.0, Offset::End, Size::Fit, Padding::default()), first, second, button)
    }
}

#[derive(Debug, Component)]
pub struct SettingsButton(Stack, IconButton, #[skip] String);

impl OnEvent for SettingsButton {}

impl SettingsButton {
    pub fn new(id: String, mut button: IconButton) -> Self {
        button.set_trigger_on_press(false);
        SettingsButton(Stack::default(), button, id)
    }

    pub fn id(&self) -> String {
        self.2.clone()
    }

    pub fn inner(&mut self) -> &mut IconButton {
        &mut self.1
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
        let state = if selected {ButtonState::Selected} else {ButtonState::Default};
        IconButton::new(
            ctx,
            icon,
            ButtonSize::Large,
            ButtonStyle::Secondary,
            state,
            Box::new(on_click),
            None,
        )
    }
}