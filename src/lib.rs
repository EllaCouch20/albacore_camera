use pelican_ui::{include_assets, Theme, Component, Context, Plugins, Plugin, maverick_start, start, Application, PelicanEngine, MaverickOS};
use pelican_ui::drawable::{Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::OnEvent;
use pelican_ui::runtime::{Services, ServiceList};
use pelican_ui::hardware::ApplicationSupport;
use profiles::plugin::ProfilePlugin;
use profiles::service::{ProfileService};
use pelican_ui_std::{Stack, Interface};


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
use service::MyPhotos;
mod components;
mod events;
mod pages;
use pages::CameraHome;

pub struct MyApp;
impl Services for MyApp {
    fn services() -> ServiceList {
        let mut services = ServiceList::default();
        services.insert::<ProfileService>();
        services.insert::<LensService>();
        services
        // ServiceList(BTreeMap::new())
    }
}

impl Plugins for MyApp {
    fn plugins(ctx: &mut Context) -> Vec<Box<dyn Plugin>> {
        vec![Box::new(ProfilePlugin::new(ctx)), Box::new(LensPlugin::new(ctx))]
        // vec![]
    }
}

impl Application for MyApp {
    async fn new(ctx: &mut Context) -> Box<dyn Drawable> { 
        ctx.assets.include_assets(include_assets!("./resources"));
        let mut theme = Theme::default(&mut ctx.assets);
        theme.colors.button.secondary_default.background = theme.colors.background.primary;
        theme.colors.button.secondary_hover.background = theme.colors.outline.secondary;
        theme.brand.illustrations.insert(ctx, "test", "images/darkglass_logo.png");
        ctx.theme = theme;
        App::new(ctx) 
    }
}

start!(MyApp);

#[derive(Debug, Component)]
pub struct App(Stack, Interface);

impl App {
    pub fn new(ctx: &mut Context) -> Box<Self> {
        let storage_path = ApplicationSupport::get().unwrap();
        std::fs::create_dir_all(&storage_path).unwrap();
        let path = storage_path.join("photos.json");
        let photos = Self::load_photos(&path);
 
        ctx.state().set(MyPhotos(photos));

        let home = CameraHome::new(ctx, None);
        let interface = Interface::new(ctx, Box::new(home), None, None);
        Box::new(App(Stack::default(), interface))
    }

    pub fn save_photos<P: AsRef<Path>>(path: P, photos: &Vec<String>) {
        let path = path.as_ref();
        let bytes = serde_json::to_vec_pretty(photos).expect("Could not vec to pretty");

        let mut tmp = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new("."))).expect("Could not write temp");
        tmp.write_all(&bytes).expect("Could not write all");
        tmp.flush().expect("Could not flush");
        tmp.persist(path).expect("Colud not persist");
    }

    pub fn load_photos<P: AsRef<Path>>(path: P) -> Vec<String> {
        let path = path.as_ref();
        if !path.exists() {
            return Vec::new();
        }
        let file = File::open(path).expect("Could not open path");
        let reader = BufReader::new(file);
        let contracts = serde_json::from_reader(reader).expect("Could not read from reader");
        contracts
    }
}

impl OnEvent for App {}
