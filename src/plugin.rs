// use pelican_ui::air::Id;
use pelican_ui::runtime;
use pelican_ui::{Context, Plugin};
// use serde_json::{Value, json};
// use std::hash::{DefaultHasher, Hasher, Hash};

use crate::service::{LensRequest, LensService};

pub struct LensPlugin(runtime::Context);
impl Plugin for LensPlugin {
    fn new(ctx: &mut Context) -> Self {
        LensPlugin(ctx.runtime.clone())
    }
}
impl LensPlugin {
    pub fn request(&mut self, request: LensRequest) {
        println!("SENDING REQUEST {:?}", request);
        self.0.send::<LensService>(&request)
    }

    // pub fn create_message(ctx: &mut Context, id: Id, message: Message) {
    //     let mut guard = ctx.get::<LensPlugin>();
    //     let plugin = guard.get().0;
    //     plugin.request(RoomsRequest::CreateMessage(id, message));
    // }
}
