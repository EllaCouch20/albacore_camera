use pelican_ui::{include_assets, Theme, Component, Context, Plugins, Plugin, maverick_start, start, Application, PelicanEngine, MaverickOS};
use pelican_ui::drawable::{Color, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{OnEvent, Event};
use pelican_ui::runtime::{Services, ServiceList};
use pelican_ui::hardware::ApplicationSupport;
use pelican_ui_std::{Stack, Interface, EncodedImage};


use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::io::BufReader;
use tempfile::NamedTempFile;

// mod bdk;
// use bdk::BDKPlugin;
// mod msg;
// use msg::MSGPlugin;
// use ucp_rust::UCPPlugin;

mod plugin;
use plugin::LensPlugin;
mod service;
use service::LensService;
use service::MyCameraRoll;
mod events;
use events::TakePhotoEvent;
mod pages;
use pages::CameraHome;
mod camera;
mod layout;
mod components;

pub struct MyApp;
impl Services for MyApp {
    fn services() -> ServiceList {
        let mut services = ServiceList::default();
        services.insert::<LensService>();
        services
        // ServiceList(BTreeMap::new())
    }
}

impl Plugins for MyApp {
    fn plugins(ctx: &mut Context) -> Vec<Box<dyn Plugin>> {
        vec![Box::new(LensPlugin::new(ctx))]
        // vec![]
    }
}

impl Application for MyApp {
    async fn new(ctx: &mut Context) -> Box<dyn Drawable> { 
        ctx.assets.include_assets(include_assets!("./resources"));
        let mut theme = Theme::default(&mut ctx.assets);

        theme.colors.background.primary = Color::from_hex("081B2F", 210); 
        theme.colors.background.secondary = Color::from_hex("081B2F", 255);
        theme.colors.outline.secondary = Color::from_hex("0F365E", 255);
        theme.colors.shades.lighten = Color::from_hex("081B2F", 120); 

        theme.colors.brand.primary = Color::from_hex("EC2149", 255);

        theme.colors.button.primary_default.background = Color::from_hex("EC2149", 255);
        theme.colors.button.primary_hover.background = Color::from_hex("E0143D", 255);
        theme.colors.button.primary_selected.background = Color::from_hex("E0143D", 255);
        theme.colors.button.primary_pressed.background = Color::from_hex("E0143D", 255);
        // theme.colors.button.ghost_default.background = Color::from_hex("000000", 0);
        // theme.colors.button.ghost_hover.background = Color::from_hex("000000", 0);

        theme.colors.button.ghost_selected.background = Color::from_hex("000000", 0);
        theme.colors.button.ghost_pressed.background = Color::from_hex("000000", 0);

        theme.colors.button.secondary_default.background = Color::from_hex("000000", 0);
        theme.colors.button.secondary_default.outline = Color::from_hex("FFFFFF", 255);
        
        theme.colors.button.secondary_hover.background = Color::from_hex("EC2149", 255);
        theme.colors.button.secondary_hover.outline = Color::from_hex("FFFFFF", 0);
        theme.colors.button.secondary_selected.background = Color::from_hex("EC2149", 255);
        theme.colors.button.secondary_selected.outline = Color::from_hex("FFFFFF", 0);
        theme.colors.button.secondary_pressed.background = Color::from_hex("E0143D", 255);
        theme.colors.button.secondary_pressed.outline = Color::from_hex("FFFFFF", 0);

        let icons = vec![
            "brightness", "camera_roll", "contrast", 
            "exposure_iso", "exposure_duration", 
            "gamma", "saturation",
            "share", "sliders", "temperature",
            "white_balance_r", "white_balance_g", 
            "white_balance_b", "camera_shutter",
            "camera_shutter_active", "exposure_stacking",
            "flash", "flip_camera"
        ];

        icons.into_iter().for_each(|p| theme.icons.insert(ctx, p));

        theme.brand.illustrations.insert(ctx, "blank", "images/blank.png");
        theme.brand.illustrations.insert(ctx, "focus", "illustrations/focus.svg");

        ctx.theme = theme;
        App::new(ctx) 
    }
}

start!(MyApp);

#[derive(Debug, Component)]
pub struct App(Stack, Interface, #[skip] bool);

impl App {
    pub fn new(ctx: &mut Context) -> Box<Self> {
        let storage_path = ApplicationSupport::get().unwrap();
        std::fs::create_dir_all(&storage_path).unwrap();
        let path = storage_path.join("my_camera_roll.json");
        
        let photos = Self::load_photos(&path);
        ctx.state().set(MyCameraRoll(photos.clone()));
        let rgbas = photos.into_iter().map(|(p, _)| EncodedImage::decode_rgba(&p)).collect();

        let home = CameraHome::new(ctx, None, rgbas);
        let interface = Interface::new(ctx, Box::new(home), None, None);
        Box::new(App(Stack::default(), interface, false))
    }

    pub fn load_photos<P: AsRef<Path>>(path: P) -> Vec<(String, (f32, f32))> {
        let path = path.as_ref();
        if !path.exists() { return Vec::new(); }
        let file = File::open(path).expect("Could not open path");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Could not read from reader")
    }
}

impl OnEvent for App {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if event.downcast_ref::<TakePhotoEvent>().is_some() {
            let storage_path = ApplicationSupport::get().unwrap();
            std::fs::create_dir_all(&storage_path).unwrap();
            let path = storage_path.join("my_camera_roll.json");
            let photos = ctx.state().get_or_default::<MyCameraRoll>().0.clone();
            let bytes = serde_json::to_vec_pretty(&photos).expect("Could not vec to pretty");
            let mut tmp = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new("."))).expect("Could not write temp");
            tmp.write_all(&bytes).expect("Could not write all");
            tmp.flush().expect("Could not flush");
            tmp.persist(path).expect("Colud not persist");
        }
        true
    }
}
