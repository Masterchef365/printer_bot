mod pos58;
use pos58::POS58USB;

use log::{error, info, warn};
use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread;
mod parsing;
use escposify::printer::Printer;
use escposify::img::Image;
use std::io::Write;
use image::{GenericImage, ImageBuffer, RgbImage};
use dither::ditherer::Dither;

const TOKEN: &str = "NTg4ODk1ODU0NTk3NzY3MjE0.XQNNtw.fB_4QmMCQzt3bqv5HLiONkF4FBY";

/*
fn main () {
    let algo = dither::ditherer::FLOYD_STEINBERG;
    let palette = [
        dither::color::RGB::from([0, 0, 0]),
        dither::color::RGB::from([255, 255, 255]),
    ];
    let img: dither::prelude::Img<dither::color::RGB<f64>> = dither::prelude::Img::load("~/Downloads/Dank.png").unwrap();
    let quant = dither::color::palette::quantize(&palette);
    let a = algo.dither(img, quant);
    a.convert_with(|x| u8::from(x)).save(std::path::Path::new("~/YES.png"));
}

fn mmain() {
    let mut usb_context = libusb::Context::new().unwrap();
    let mut pos = POS58USB::new(&mut usb_context, std::time::Duration::from_secs(1)).unwrap();
    let mut printer = Printer::new(&mut pos, None, None);
    let img = ImageBuffer::from_fn(20, 20, |x, _| {
        if x % 2 == 0 {
            image::Rgb([0, 0, 0])
        } else {
            image::Rgb([0xFF, 0xFF, 0xFF])
        }
    });
    let img = image::DynamicImage::ImageRgb8(img);
    let img = Image::from(img);
    /*
    let _ = printer
        .align("lt")
        .bit_image(&img, Some("s8"))
        .bit_image(&img, Some("d8"))
        .bit_image(&img, Some("s24"))
        .bit_image(&img, Some("d24"));
        */
}
*/

fn main() {
    let _ = env_logger::builder()
        .filter(Some("print_bot"), log::LevelFilter::Trace)
        .try_init();

    // Create sender and receiver pair for messages
    let (sender, receiver) = channel::<String>();

    // Consumer thread for message contents
    info!("Spawning consumer thread");
    thread::spawn(move || consumer_thread(receiver));

    // Create bot thread
    info!("Starting bot thread");
    let handler = Handler::new(sender);
    let mut client = Client::new(&TOKEN, handler).expect("Err creating client");

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}

fn consumer_thread(receiver: Receiver<String>) {
    loop {
        if let Ok(mut usb_context) = libusb::Context::new() {
            info!("New USB context created");
            if let Ok(mut pos) = POS58USB::new(&mut usb_context, std::time::Duration::from_secs(90))
            {
                //let mut printer = Printer::new(&mut pos, None, None);
                info!("Connected to printer.");
                'inner: loop {
                    info!("Waiting for messages");
                    if let Ok(message) = receiver.recv() {
                        info!("Printing a message.");
                        let mut filtered: String = message.chars().filter(|x| x.is_ascii()).collect();
                        filtered.push('\n');
                        if let Err(e) = pos.write(filtered.as_bytes()) {
                            warn!("Could not write to printer. ({})", e);
                            break 'inner;
                        }
                    } else {
                        warn!("Receiver could not receive");
                    }
                }
            } else {
                warn!("Cannot connect to printer");
                thread::sleep(std::time::Duration::from_secs(5))
            }
        } else {
            warn!("Could not create USB context");
            thread::sleep(std::time::Duration::from_secs(5))
        }
    }
}

struct Handler {
    sender: Mutex<Sender<String>>,
}

impl Handler {
    pub fn new(sender: Sender<String>) -> Self {
        Self {
            sender: Mutex::new(sender),
        }
    }

    fn print(&self, message: &str) {
        if let Ok(sender) = self.sender.lock() {
            let mut message = String::from(message);
            message.push('\n');
            sender
                .send(message)
                .expect("Sender endpoint is not connected");
        }
    }
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        if let Some(print_content) = parsing::parse_message(&msg.content) {
            info!("Sending a print job.");

            self.print(print_content);
            if let Err(why) = msg
                .channel_id
                    .say(ctx.http, "Received. Sending message to server.")
                    {
                        warn!("Error sending message: {:?}", why);
                    }
        }
    }

    fn ready(&self, _: Context, _: Ready) {
        info!("Bot thread finished connecting");
    }
}
