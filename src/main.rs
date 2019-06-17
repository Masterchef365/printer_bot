mod handling;
mod parsing;
mod pos58;
use escposify::printer::Printer;
use log::{error, info};
use pos58::POS58USB;
use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread;

const TOKEN: &str = "NTg4ODk1ODU0NTk3NzY3MjE0.XQNNtw.fB_4QmMCQzt3bqv5HLiONkF4FBY";

fn main() {
    let _ = env_logger::builder()
        .filter(Some("print_bot"), log::LevelFilter::Trace)
        .try_init();

    // Create sender and receiver pair for messages
    let (sender, receiver) = channel::<Message>();

    // Consumer thread for message contents
    info!("Spawning consumer thread");
    thread::spawn(move || consumer_thread(receiver));

    // Create bot thread
    info!("Starting bot thread");
    let handler = Handler::new(sender);
    let mut client = Client::new(&TOKEN, handler).expect("Error creating client");

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}

fn consumer_thread(receiver: Receiver<Message>) {
    let mut usb_context = libusb::Context::new().expect("Failed to create LibUSB context.");
    let mut device = POS58USB::new(&mut usb_context, std::time::Duration::from_secs(90))
        .expect("Failed to connect to printer");
    let mut printer = Printer::new(&mut device, None, None);
    let reqwest_client = reqwest::Client::new();
    loop {
        handling::handle_message(receiver.recv().unwrap(), &reqwest_client, &mut printer);
    }
}

struct Handler {
    sender: Mutex<Sender<Message>>,
}

impl Handler {
    pub fn new(sender: Sender<Message>) -> Self {
        Self {
            sender: Mutex::new(sender),
        }
    }
}

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {
        if let Ok(sender) = self.sender.lock() {
            sender.send(msg).expect("Sender endpoint is not connected");
        }
    }

    fn ready(&self, _: Context, _: Ready) {
        info!("Bot thread finished connecting");
    }
}
