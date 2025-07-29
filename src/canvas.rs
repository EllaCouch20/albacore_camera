use rust_on_rails::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;

#[derive(Default)]
pub struct MyBackgroundTask;
#[async_trait]
impl Task for MyBackgroundTask {
    fn interval(&self) -> Option<Duration> {Some(Duration::from_secs(1))}
    async fn run(&mut self, ctx: &mut HeadlessContext) {
        let Modify(modify) = ctx.cache.get::<Modify>().await;
        println!("background tick: {modify}");
        let CacheCount(cc) = ctx.cache.get::<CacheCount>().await;
        ctx.cache.set(&CacheCount(cc+modify)).await;
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Count(i32);

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Modify(i32);

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct CacheCount(i32);

pub struct Channel<I, O>(Receiver<I>, Sender<O>);

impl<I, O> Channel<I, O> {
    pub fn new() -> (Self, Channel<O, I>) {
        let (si, ri) = channel::<I>();
        let (so, ro) = channel::<O>();
        (Channel(ri, so), Channel(ro, si))
    }
}

pub struct MyTask(Channel<i32, i32>);

#[async_trait]
impl Task for MyTask {
    fn interval(&self) -> Option<Duration> {Some(Duration::from_secs(1))}

    async fn run(&mut self, ctx: &mut HeadlessContext) {
        println!("running task");
        let Modify(modify) = ctx.cache.get::<Modify>().await;
        let modify = modify + self.0.0.try_iter().sum::<i32>();
        ctx.cache.set(&Modify(modify)).await;
        let CacheCount(cc) = ctx.cache.get::<CacheCount>().await;
        self.0.1.send(cc).unwrap();
    }
}

pub struct MyApp {
    channel: Channel<i32, i32>,
    font: Font
}

impl App for MyApp {
    async fn background_tasks(_ctx: &mut HeadlessContext) -> Tasks {
        tasks![MyBackgroundTask]
    }
    async fn new(ctx: &mut Context<'_>) -> (Self, Tasks) {
        let font = ctx.add_font(include_bytes!("../resources/fonts/outfit_regular.ttf"));
        let (channel, input) = Channel::<i32, i32>::new();
        (MyApp{channel, font}, tasks![MyTask(input)])
    }

    fn on_event(&mut self, ctx: &mut Context<'_>, event: Event) {
        match event {
            Event::Tick => {
                if let Some(latest) = self.channel.0.try_iter().last() {
                    ctx.state().set(&Count(latest));
                }
                let text = format!("Count: {}", ctx.state().get::<Count>().0);
                ctx.clear(Color(0, 0, 255, 255));
                ctx.draw(Area((200.0, 200.0), None), CanvasItem::Shape(Shape::Rectangle(20.0, (ctx.size().0/2.0, ctx.size().1/2.0)), Color(255, 255, 0, 255)));
                ctx.draw(Area((250.0, 250.0), None), CanvasItem::Text(Text::new(
                    None, vec![Span::new(&text, 48.0, 60.0, self.font.clone(), Color(255, 0, 0, 255))], None, Align::Left)
                ));
            },
            Event::Mouse{..} => self.channel.1.send(-10).unwrap(),
            Event::Keyboard{..} => self.channel.1.send(100).unwrap(),
            _ => {}
        }
    }
}

create_entry_points!(MyApp);
