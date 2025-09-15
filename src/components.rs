use pelican_ui::{Component, Context};
use pelican_ui::drawable::{Drawable, Component, Image, Align, ShapeType};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{Event, OnEvent, TickEvent, MouseEvent, MouseState};

// use crate::pages::CameraRoll;
use crate::events::{ResetSetting, TakePhotoEvent, SettingsSelect, OpenSettingsEvent, EnumeratorSelectEvent, SelectImageEvent};
use crate::MyCameraRoll;
use image::RgbaImage;

use pelican_ui_std::{
    Stack, EncodedImage, Padding,
    Bin, Rectangle, Size,
    Offset, RoundedRectangle, Icon,
    ButtonState, IconButton,
    ButtonSize, ButtonStyle,
    AdjustScrollEvent, Row,
    ScrollAnchor, Scroll,
    Column, TextStyle,
    ExpandableText, Slider,
    Button, ButtonWidth,
    Wrap, ExpandableImage,
    NavigateEvent
};

pub type SettingClosure = Box<dyn FnMut(&mut Context, f32)>;

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
                self.1 = Image{shape: ShapeType::RoundedRectangle(0.0, (48.0, 48.0), 8.0, 0.0), image, color: None};
                self.3 = photos.len();
            }
        } else if let Some(MouseEvent { state: MouseState::Pressed, position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
            ctx.trigger_event(NavigateEvent(0))
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
pub struct SettingsBumper(
    Column, SettingsOptions, 
    Bin<Stack, Rectangle>, 
    Option<SettingsDetails>, 
    Option<EnumeratorSelector>, 
    Option<SettingsDetails>, 
    Option<Slider>, 
    Option<SettingsDetails>, 
    Option<Slider>
);

type SettingsSliderData<'a> = (Box<dyn FnMut(&mut Context, f32)>, f32, &'a str);
type SettingsEnumeratorData<'a> = (Vec<(&'a str, Box<dyn FnMut(&mut Context)>)>, &'a str, usize);

impl SettingsBumper {
    pub fn new(ctx: &mut Context, closure: SettingClosure, val: f32, label: &str) -> Self {
        let color = ctx.theme.colors.outline.secondary;
        let layout = Stack(Offset::Center, Offset::Center, Size::fill(), Size::Static(3.0), Padding::default());
        SettingsBumper(
            Column::new(16.0, Offset::Start, Size::fill(), Padding::default()),
            SettingsOptions::new(ctx),
            Bin(layout, Rectangle::new(color, 0.0)),
            None,
            None,
            Some(SettingsDetails::new(ctx, label)),
            Some(Slider::new(ctx, val, None, None, closure)),
            None,
            None,
        )
    }

    pub fn set_first_slider(&mut self, ctx: &mut Context, 
        slider_a: Option<SettingsSliderData>,
    ) {
        match slider_a {
            Some((closure, val, label)) => {
                self.5 = Some(SettingsDetails::new(ctx, label));
                self.6 = Some(Slider::new(ctx, val, None, None, closure));
            },
            None => {
                self.5 = None;
                self.6 = None;
            }
        }
    }

    pub fn set_second_slider(&mut self, ctx: &mut Context,
        slider_b: Option<SettingsSliderData>,
    ) {
        match slider_b {
            Some((closure, val, label)) => {
                self.7 = Some(SettingsDetails::new(ctx, label));
                self.8 = Some(Slider::new(ctx, val, None, None, closure));
            },
            None => {
                self.7 = None;
                self.8 = None;
            }
        }
    }

    pub fn set_enumerator(&mut self, ctx: &mut Context, enumerator: Option<SettingsEnumeratorData>) {
        match enumerator {
            Some((items, label, index)) => {
                self.3 = Some(SettingsDetails::new(ctx, label));
                self.4 = Some(EnumeratorSelector::new(ctx, items, index));
            },
            None => {
                self.3 = None;
                self.4 = None;
            }
        }
    }
}

impl OnEvent for SettingsBumper {}

#[derive(Debug, Component)]
pub struct SettingsOptions(Scroll, SettingsOptionsContent);

impl SettingsOptions {
    pub fn new(ctx: &mut Context) -> Self {
        let width = Size::custom(move |_| (0.0, f32::MAX));
        let layout = Scroll::horizontal(Offset::Start, Offset::Start, width, Size::Fit, Padding::default(), ScrollAnchor::Start);
        SettingsOptions(layout, SettingsOptionsContent::new(ctx)) 
    }
}

impl OnEvent for SettingsOptions {
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(AdjustScrollEvent::Horizontal(a)) = event.downcast_ref::<AdjustScrollEvent>() {
            self.0.adjust_scroll(*a);
        } else if let Some(MouseEvent { state: MouseState::Scroll(x, _), position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
            self.0.adjust_scroll(*x);
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct SettingsOptionsContent(Row, Vec<SettingsButton>);

impl SettingsOptionsContent {
    pub fn new(ctx: &mut Context) -> Self {
        let icons = vec!["brightness", "exposure"];

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

// pub exposure_mode: ExposureMode,
// pub custom_exposure: Option<CustomExposure>, // duration + ISO
// pub exposure_compensation: Option<f32>,      // EV
// pub focus_mode: FocusMode,
// pub focus_distance: Option<f32>,            // 0.0..1.0 lens position for manual
// pub focus_point_of_interest: Option<(f32,f32)>, // normalized x,y for focus
// pub white_balance_mode: WhiteBalanceMode,
// pub white_balance_gains: Option<WhiteBalanceGains>,
// pub torch_enabled: bool,
// pub zoom_factor: Option<f32>,
// pub frame_rate: Option<f32>,
// pub resolution: Option<Resolution>,
// pub hdr_enabled: bool,
// pub stabilization_enabled: bool,
// pub low_light_boost: Option<bool>,
// pub scene_mode_hint: Option<SceneMode>,
// pub brightness: Option<f32>,
// pub contrast: Option<f32>,
// pub saturation: Option<f32>,
// pub sharpness: Option<f32>,
// pub hue: Option<f32>,
// pub noise_reduction: Option<f32>,
// pub gamma: Option<f32>,
// pub color_filter: Option<ColorFilter>,

#[derive(Debug, Component)]
pub struct SettingsDetails(Row, ExpandableText, Button);

impl OnEvent for SettingsDetails {}

impl SettingsDetails {
    pub fn new(ctx: &mut Context, label: &str) -> Self {
        let font_size = ctx.theme.fonts.size;
        let label = ExpandableText::new(ctx, label, TextStyle::Primary, font_size.h5, Align::Left, None);
        let button = Button::new(ctx, None, None, Some("Reset"), None,
            ButtonSize::Medium, ButtonWidth::Hug, ButtonStyle::Primary,
            ButtonState::Default, Offset::Center, |ctx: &mut Context| {ctx.trigger_event(ResetSetting)}, None
        );
        SettingsDetails(Row::new(6.0, Offset::End, Size::Fit, Padding::default()), label, button)
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
    #[allow(clippy::new_ret_no_self)]
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


#[derive(Debug, Component)]
pub struct EnumeratorSelector(Row, Vec<EnumeratorSelectorButton>);

type EnumeratorSelectorData<'a> = Vec<(&'a str, Box<dyn FnMut(&mut Context)>)>;

impl EnumeratorSelector {
    pub fn new(ctx: &mut Context, items: EnumeratorSelectorData, index: usize) -> Self {
        let buttons = items.into_iter().enumerate().map(|(idx, (item, mut on_click))| {

            let id = item.to_string();
            let closure = move |ctx: &mut Context| {
                ctx.trigger_event(EnumeratorSelectEvent(id.clone()));
                on_click(ctx)
            };
            EnumeratorSelectorButton::new(ctx, item.to_string(), item, index == idx, closure)
        }).collect();
        EnumeratorSelector(Row::center(24.0), buttons)
    }
}

impl OnEvent for EnumeratorSelector {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(EnumeratorSelectEvent(id)) = event.downcast_ref::<EnumeratorSelectEvent>() {
            self.1.iter_mut().for_each(|button| {
                let status = if button.id() == *id {ButtonState::Selected} else {ButtonState::Default};
                *button.inner().status() = status;
                button.inner().color(ctx);
            });
        }
        true
    }
}

#[derive(Debug, Component)]
struct EnumeratorSelectorButton(Stack, Button, #[skip] String);
impl OnEvent for EnumeratorSelectorButton {}

impl EnumeratorSelectorButton {
    fn new(
        ctx: &mut Context, id: String, label: &str,
        selected: bool, on_click: impl FnMut(&mut Context) + 'static
    ) -> Self {
        let button = Self::new_button(ctx, label, selected, on_click);
        EnumeratorSelectorButton(Stack::default(), button, id)
    }

    pub fn new_button(
        ctx: &mut Context, 
        label: &str,
        selected: bool,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Button {
        let state = if selected {ButtonState::Selected} else {ButtonState::Default};
        Button::new(
            ctx,
            None,
            None,
            Some(label),
            None,
            ButtonSize::Medium,
            ButtonWidth::Hug,
            ButtonStyle::Secondary,
            state,
            Offset::Center,
            on_click,
            None,
        )
    }

    fn id(&self) -> String {
        self.2.clone()
    }

    fn inner(&mut self) -> &mut Button {
        &mut self.1
    }
}


#[derive(Debug, Component)]
pub struct PhotoWrap(Box<dyn Layout>, Vec<ImageButton>, Option<ExpandableText>);
impl OnEvent for PhotoWrap {}

impl PhotoWrap {
    pub fn new(ctx: &mut Context, photos: Vec<RgbaImage>) -> Self {
        let text_size = ctx.theme.fonts.size.md;
        let help_text = photos.is_empty().then_some(ExpandableText::new(
            ctx, "Your camera roll is empty.\nTake a photo to get started.", 
            TextStyle::Primary, text_size, Align::Center, None
        ));

        let layout = match photos.is_empty() {
            true => Box::new(Stack::center()) as Box<dyn Layout>,
            false => Box::new(Wrap::new(8.0, 8.0)) as Box<dyn Layout>
        };

        let my_photos = photos.into_iter().map(|p| 
            ImageButton::new(ctx, p)
        ).collect();

        PhotoWrap(layout, my_photos, help_text)
    }
}

#[derive(Debug, Component)]
pub struct ImageButton(Stack, ExpandableImage, #[skip] RgbaImage);
impl OnEvent for ImageButton {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(MouseEvent{state: MouseState::Pressed, position: Some(_)}) = event.downcast_ref::<MouseEvent>() {
            ctx.hardware.haptic();
            ctx.trigger_event(SelectImageEvent(self.2.clone()));
            ctx.trigger_event(NavigateEvent(1));
        }
        true
    }
}

impl ImageButton {
    pub fn new(ctx: &mut Context, image: RgbaImage) -> Self {
        ImageButton(
            Stack(Offset::Center, Offset::Center, Size::Static(64.0), Size::Static(64.0), Padding::default()), 
            ExpandableImage::new(ctx.assets.add_image(image.clone()), None), image
        )
    }
}