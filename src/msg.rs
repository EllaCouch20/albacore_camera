// use messages::{Room, Message};
// use profiles::service::{Profile, Profiles};
// use pelican_ui_std::Timestamp;
// use pelican_ui::{Context, Plugin};
// use pelican_ui::air::OrangeName;

// use std::sync::{Mutex, Arc};
// use std::time::Duration;
// use serde::{Serialize, Deserialize};
// use std::collections::BTreeMap;
// use chrono::{DateTime, Local, Utc}; // temp
// use std::sync::mpsc::{channel, Receiver};

// pub struct MSGPlugin {
//     // rooms: Arc<Mutex<Vec<Room>>>,
//     profiles: Arc<Mutex<Vec<Profile>>>,
//     receiver: Receiver<String>,
// }

// impl MSGPlugin {
//     pub async fn _init(&mut self) {
//         println!("Initialized MSG");
//     }

//     // pub fn get_rooms(&mut self) -> Vec<Room> {
//     //     self.rooms.lock().unwrap().to_vec()
//     // }

//     // pub fn create_room(&mut self, profiles: Vec<Profile>) {
//     //     self.rooms.lock().unwrap().to_vec().push(Room::new(profiles, Vec::new()));
//     // }

//     pub fn get_profiles(&mut self) -> Vec<Profile> {
//         self.profiles.lock().unwrap().to_vec()
//     }

//     // get all profiles
//     // create room (members shared with?)
//     // read room (room) -> (Vec<Message>)
//     // get rooms -> (Vec<Room>)
//     // new message (msg, room)

//     // read user Profile
// }

// // let (sender, receiver) = mpsc::channel();
// // (
// //     Some(IconButton::input(ctx, icon, move |_| {sender.send(0).unwrap();})),
// //     Some((receiver, Box::new(on_click) as SubmitCallback)),
// // )

// impl Plugin for MSGPlugin {
//     async fn background_tasks(_ctx: &mut HeadlessContext) -> Tasks {
//         vec![]
//     }

//     async fn new(_ctx: &mut Context, _h_ctx: &mut HeadlessContext) -> (Self, Tasks) {
//         // let rooms = Arc::new(Mutex::new(Vec::new()));
//         let profiles = Arc::new(Mutex::new(Vec::new()));
//         let (_sender, receiver) = channel();
//         (MSGPlugin{
//             // rooms: rooms.clone(),
//             profiles: profiles.clone(),
//             receiver,
//         // }, tasks![GetRooms(rooms), GetProfiles(profiles)])
//         }, tasks![GetProfiles(profiles)])
//     }
// }

// #[derive(Serialize, Deserialize, Default, Clone, Debug)]
// pub struct AllMessages(pub Vec<Message>);

// impl AllMessages {
//     pub fn new() -> Self {
//         AllMessages(fake_messages())
//     }

//     pub fn add_message(&mut self, new: String) {
//         self.0.push(Message::from(new));
//     }
// }
