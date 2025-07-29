use std::collections::BTreeMap;
use std::sync::LazyLock;
use std::time::Duration;

use maverick_os::Cache;
use pelican_ui::runtime::{Services, Service, ServiceList, ThreadContext, async_trait, self};
use pelican_ui::{hardware};
use pelican_ui::State;
use pelican_ui::air::{Id, Service as AirService, Protocol, Validation, ChildrenValidation, HeaderInfo, RecordPath, Permissions};
// use pelican_ui_std::AvatarContent;

// use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use chrono::{Utc, DateTime};
// use uuid::Uuid;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MyPhotos(pub Vec<String>);

static PHOTOS: LazyLock<Id> = LazyLock::new(|| Id::hash(&"PhotosV1".to_string()));
static PHOTO: LazyLock<Id> = LazyLock::new(|| Id::hash(&"PhotoV1".to_string()));
static MY_PHOTOS: LazyLock<Id> = LazyLock::new(|| Id::hash(&"MYPHOTOS".to_string()));

const PHOTOS_PERMISSIONS: Permissions = Permissions::new(Some((true, true)), None, BTreeMap::new());
const PHOTO_PERMISSIONS: Permissions = Permissions::new(None, None, BTreeMap::new());

static PHOTOS_PROTOCOL: LazyLock<Protocol> = LazyLock::new(|| {
    let cv = ChildrenValidation::new(vec![*PHOTO], true, true, false);
    let validation = Validation::new(Some(cv), None, BTreeMap::new(), false);
    let header = HeaderInfo::new(None, BTreeMap::new(), Vec::new());
    Protocol::new(validation, header, *PHOTOS)
});

static PHOTO_PROTOCOL: LazyLock<Protocol> = LazyLock::new(|| {
    let validation = Validation::new(None, None, BTreeMap::new(), false);
    let header = HeaderInfo::new(None, BTreeMap::new(), Vec::new());
    Protocol::new(validation, header, *PHOTO)
});

#[derive(Serialize, Deserialize, Debug)]
pub enum LensRequest {
    // CreateRoom(Uuid),
    // CreateAlbum,
    SavePhoto(String),
    // Share(Id, OrangeName),
}

#[derive(Debug)]
pub struct LensService{
}

impl Services for LensService {
    fn services() -> ServiceList {
        let mut services = ServiceList::default();
        services.insert::<LensSync>();
        services
    }
}

#[async_trait]
impl Service for LensService {
    type Send = String;
    type Receive = LensRequest;

    async fn new(_hardware: &mut hardware::Context) -> Self {
        LensService{
        }
    }

    async fn run(&mut self, ctx: &mut ThreadContext<Self::Send, Self::Receive>) -> Result<Option<Duration>, runtime::Error> {
        // println!("RUNNING SERVICE");
        let cache = &mut LensCache::from_cache(&mut ctx.hardware.cache).await;
        while let Some((_, request)) = ctx.get_request() {
            match request {
            //     LensRequest::CreateRoom(uuid) => {
            //         while let (_, Some(_)) = AirService::create_private(ctx, RecordPath::root(), ROOMS_PROTOCOL.clone(), cache.albums_idx, ROOMS_PERMISSIONS, serde_json::to_vec(&uuid)?).await? {
            //             cache.albums_idx += 1;
            //         }
            //     },
                // LensRequest::CreateAlbum => {
                //     AirService::create_private(ctx, RecordPath::root(), PHOTOS_PROTOCOL.clone(), cache.albums_idx, PHOTOS_PERMISSIONS, serde_json::to_vec(&*MY_PHOTOS)?).await?;
                // },
                LensRequest::SavePhoto(data) => {
                    println!("Saving photo...");
                    ctx.callback(data);
                    // let mut x = cache.albums.get(&RecordPath::root().join(*MY_PHOTOS)).unwrap().1;
                    // while let (_, Some(_)) = AirService::create_private(ctx, RecordPath::root().join(*MY_PHOTOS), PHOTO_PROTOCOL.clone(), x, PHOTO_PERMISSIONS, serde_json::to_vec(&data)?).await? {
                    //     x += 1;
                    // }
                },
            //     LensRequest::Share(room, name) => {
            //         let message = Message::invisible(name.clone());
            //         let path = RecordPath::root().join(room);
            //         AirService::share(ctx, name, ROOMS_PERMISSIONS, path).await?;
            //         let mut x = cache.rooms.get(&RecordPath::root().join(room)).unwrap().2;
            //         while let (_, Some(_)) = AirService::create_private(ctx, RecordPath::root().join(room), MESSAGES_PROTOCOL.clone(), x, MESSAGES_PERMISSIONS, serde_json::to_vec(&message)?).await? {
            //             x += 1;
            //         }
            //     },
            }
        }

        Ok(Some(Duration::from_millis(16)))
    }

    fn callback(state: &mut State, response: Self::Send) {
        println!("Callback with {:?}", response);
        let mut photos = state.get::<MyPhotos>().unwrap().0.clone();
        photos.push(response);
        state.set(MyPhotos(photos));
        // let mut rooms = state.get::<Rooms>().0;
        // // if response.2 {state.set(&Name(Some(response.0.clone())));}
        // rooms.insert(response.0, response.1);
        // state.set(&Rooms(rooms));
        // let mut contracts = state.get::<MyContracts>().0;
        // contracts.insert(contract);
        // state.set(MyContracts(contract));
    }
}

#[derive(Debug)]
pub struct LensSync{
    cache: LensCache,
    init: bool 
}

impl Services for LensSync {}

#[async_trait]
impl Service for LensSync {
    type Send = Vec<String>;
    type Receive = ();

    async fn new(hardware: &mut hardware::Context) -> Self {
        LensSync{
            cache: LensCache::from_cache(&mut hardware.cache).await,
            init: false
        }
    }

    async fn run(&mut self, ctx: &mut ThreadContext<Self::Send, Self::Receive>) -> Result<Option<Duration>, runtime::Error> {
        let mut mutated = false;
        println!("running {:?}", self.cache.albums_idx);

        // for (_, path) in AirService::receive(ctx, self.cache.datetime).await?.into_iter() {
        //     // let uuid: Uuid = serde_json::from_slice(&AirService::read_private(ctx, path.clone()).await?.unwrap().0.payload).unwrap();
        //     // self.cache.rooms.entry(path).or_insert((uuid, vec![], 0));
        //     // mutated = true;

        //     println!("Creating pointer.");
        //     let mut x = self.cache.albums_idx;
        //     while let (_, Some(_)) = AirService::create_pointer(ctx, RecordPath::root(), path.clone(), x).await? {
        //         x += 1;
        //     }
        //     println!("Done creating pointers.");
        //     mutated = true;
        // }

        // // println!("Done receiving.");

        // self.cache.datetime = chrono::Utc::now();

        // while let (path, Some(_)) = AirService::discover(ctx, RecordPath::root().join(*PHOTOS), self.cache.albums_idx, vec![PHOTOS_PROTOCOL.clone()]).await? {
        //     println!("Discovering...");
        //     if let Some(path) = path {
        //         if let Ok(data) = serde_json::from_slice::<Vec<String>>(&AirService::read_private(ctx, path.clone()).await?.unwrap().0.payload) {
        //             println!("Photo found...");
        //             self.cache.albums.entry(path).or_insert((data, 0));
        //             mutated = true;
        //         } else {println!("_--- PATH HAD NO PHOTO ---_");}
        //     }
        //     self.cache.albums_idx += 1;
        // }
        // println!("Done discovering.");

        for (room, (photos, index)) in &mut self.cache.albums {
            while let (path, Some(_)) = AirService::discover(ctx, room.clone(), *index, vec![PHOTO_PROTOCOL.clone()]).await? {
                if let Some(path) = path {
                    if let Ok(data) = serde_json::from_slice(&AirService::read_private(ctx, path).await?.unwrap().0.payload) {
                        photos.insert(*index as usize, data);
                        mutated = true;
                    }
                }
                *index += 1;
            }
        }

        println!("Done messages.");
        
        if mutated || !self.init {
            self.init = true;
            ctx.callback(self.cache.albums.iter().map(|(_, (data, _))| {
                data.clone()
            }).collect::<Vec<Vec<String>>>().into_iter().flatten().collect());
            println!("Callback done.");
        }

        // println!("Done updating.");
        self.cache.cache(&mut ctx.hardware.cache).await;
        // println!("DONE");
        Ok(Some(Duration::from_secs(1)))
    }

    fn callback(state: &mut State, response: Self::Send) {
        state.set(MyPhotos(response))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LensCache {
    pub albums_idx: u32,
    pub albums: BTreeMap<RecordPath, (Vec<String>, u32)>,
    pub datetime: DateTime<Utc>,
}

impl LensCache {
    pub async fn cache(&self, cache: &mut Cache) {
        cache.set("LensCache", self).await;
    }

    pub async fn from_cache(cache: &mut Cache) -> Self {
        cache.get("LensCache").await
    }

}

impl Default for LensCache {
    fn default() -> Self {
        println!("LensCache as default");
        LensCache {
            albums_idx: 0,
            albums: BTreeMap::new(),
            datetime: DateTime::UNIX_EPOCH,
        }
    }
}
