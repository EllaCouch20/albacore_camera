use pelican_ui::{Component, Context};
use pelican_ui::drawable::{Drawable, Component, Image};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{Event, OnEvent, TickEvent};
use pelican_ui::hardware::{Camera, CameraSettings};

use image::RgbaImage;

// use crate::pages::CameraRoll;
use crate::service::LensRequest;
use crate::LensPlugin;
use crate::events::TakePhotoEvent;
use crate::layout::SpaceEvenly;
use crate::events::SetSettingEvent;

use pelican_ui_std::{
    Stack, ExpandableImage,
    EncodedImage, Padding,
    Bin, Rectangle, Size,
    Offset, AspectRatioImage
};

#[derive(Debug, Component)]
pub struct AlbacoreCamera(Stack, ExpandableImage, ThirdsGrid, FocusIndicator, #[skip] Option<Camera>, #[skip] Option<RgbaImage>);

impl AlbacoreCamera {
    pub fn new(ctx: &mut Context) -> Self {  
        let blank = ctx.theme.brand.illustrations.get("blank").unwrap();
        AlbacoreCamera(Stack::fill(), 
            ExpandableImage::new(blank, None), 
            ThirdsGrid::new(ctx), 
            FocusIndicator::new(ctx),
            Camera::new_unprocessed().ok().map(|c| c.start()), 
            None,
        )
    }

    pub fn camera(&mut self) -> &mut Option<Camera> {&mut self.4}
}

impl OnEvent for AlbacoreCamera {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            if let Some(ref mut camera) = self.4 {
                if let Ok(raw_frame) = camera.frame() {
                    self.5 = Some(raw_frame.clone());
                    let image = ctx.assets.add_image(raw_frame);
                    self.1.image().image = image;
                }
            }
        } else if let Some(TakePhotoEvent) = event.downcast_ref::<TakePhotoEvent>() {
            if let Some(rgba) = &self.5 {
                // ctx.trigger_event(NewPhotoEvent(rgba.clone()));
                let mut guard = ctx.get::<LensPlugin>();
                let plugin = guard.get().0;
                let img = EncodedImage::encode_rgba(rgba.clone());
                plugin.request(LensRequest::SavePhoto(img, (rgba.width() as f32, rgba.height() as f32)));
            }
        } else if let Some(setting) = event.downcast_ref::<SetSettingEvent>() {
            if let Some(camera) = &mut self.4 {
                if let Some(settings_arc) = camera.settings().as_mut() {
                    let mut settings_guard = settings_arc.lock().unwrap();
                    let settings: &mut CameraSettings = &mut settings_guard;

                    match setting {
                        SetSettingEvent::Brightness(p) => settings.set_brightness(*p),
                        SetSettingEvent::ExposureMode(mode) => settings.set_exposure_mode(*mode),
                        SetSettingEvent::CustomExposureTime(dur) => {
                            let iso = settings.custom_exposure.unwrap_or_default().iso;
                            settings.set_custom_exposure(*dur, iso)
                        },
                        SetSettingEvent::CustomExposureISO(iso) => {
                            let dur = settings.custom_exposure.unwrap_or_default().duration;
                            settings.set_custom_exposure(dur, *iso)
                        },
                        _ => {}
                    }
                }
            }
        }
        true
    }
}

// set_brightness
// set_contrast
// set_saturation
// set_sharpness
// set_hue
// set_noise_reduction
// set_gamma
// set_focus_distance
// set_exposure_compensation
// set_custom_exposure
// set_white_balance_gains
// set_zoom_factor
// set_torch_enabled
// set_hdr_enabled
// set_stabilization_enabled
// set_low_light_boost
// set_scene_mode
// set_focus_point_of_interest


#[derive(Debug, Component)]
pub struct FocusIndicator(Stack, Image);
impl OnEvent for FocusIndicator {}

impl FocusIndicator {
    pub fn new(ctx: &mut Context) -> Self {
        let image = ctx.theme.brand.illustrations.get("focus").unwrap();
        let image = AspectRatioImage::new(image, (68.0, 68.0));
        FocusIndicator(Stack::fill(), image)
    }
}

#[derive(Debug, Component)]
pub struct ThirdsGrid(Stack, ThirdsGridInner, ThirdsGridInner);
impl OnEvent for ThirdsGrid {}

impl ThirdsGrid {
    pub fn new(ctx: &mut Context) -> Self {
        ThirdsGrid(Stack::fill(), ThirdsGridInner::vertical(ctx), ThirdsGridInner::horizontal(ctx))
    }
}


#[derive(Debug, Component)]
struct ThirdsGridInner(SpaceEvenly, Bin<Stack, Rectangle>, Bin<Stack, Rectangle>);
impl OnEvent for ThirdsGridInner {}

impl ThirdsGridInner {
    fn horizontal(ctx: &mut Context) -> Self {
        let color = ctx.theme.colors.shades.lighten;

        let layout = Stack(Offset::Start, Offset::Start, Size::Static(3.0), Size::fill(), Padding::default());
        let a = Bin(layout, Rectangle::new(color, 0.0));

        let layout = Stack(Offset::Start, Offset::Start, Size::Static(3.0), Size::fill(), Padding::default());
        let b = Bin(layout, Rectangle::new(color, 0.0));

        let layout = SpaceEvenly::horizontal(Offset::Start, Size::fill(), Padding::default());

        ThirdsGridInner(layout, a, b)
    }

    fn vertical(ctx: &mut Context) -> Self {
        let color = ctx.theme.colors.shades.lighten;

        let layout = Stack(Offset::Start, Offset::Start, Size::fill(), Size::Static(3.0), Padding::default());
        let a = Bin(layout, Rectangle::new(color, 0.0));

        let layout = Stack(Offset::Start, Offset::Start, Size::fill(), Size::Static(3.0), Padding::default());
        let b = Bin(layout, Rectangle::new(color, 0.0));

        let layout = SpaceEvenly::vertical(Offset::Start, Size::fill(), Padding::default());

        ThirdsGridInner(layout, a, b)
    }
}

