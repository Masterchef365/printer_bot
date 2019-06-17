use crate::parsing::*;
use dither::color::*;
use dither::ditherer::*;
use escposify::printer::Printer;
use image::{DynamicImage, FilterType, ImageBuffer};
use log::warn;
use serenity::model::channel::Message;
use std::io::Write;

pub fn handle_message<W: Write>(
    message: Message,
    reqwest_client: &reqwest::Client,
    printer: &mut Printer<W>,
) {
    if let Some(text) = parse_command_body(&message.content) {
        // Block text, print text directly
        if let Some(blocktext) = parse_blocktext(&text) {
            let filtered: String = blocktext.chars().filter(|x| x.is_ascii()).collect();
            if let Err(e) = printer.text(&filtered) {
                warn!("Printer failed to write blocktext {} ({:?})", filtered, e);
            } else {
                let _ = printer.flush();
            }
        }

        // Download and print the image at the URL url
        let mut download_and_print = move |url| match download_image(reqwest_client, url) {
            Ok(Ok(image)) => {
                if let Err(e) = print_image(printer, image) {
                    warn!("Could not print image, {:?}", e);
                } else {
                    let _ = printer.flush();
                }
            }
            _ => warn!("Could not download image"),
        };

        if let Ok(url) = reqwest::Url::parse(text) {
            download_and_print(url);
        }

        for attachment in message.attachments {
            if let Ok(url) = reqwest::Url::parse(&attachment.url) {
                download_and_print(url);
            }
        }
    }
}

fn download_image<U: reqwest::IntoUrl>(
    client: &reqwest::Client,
    url: U,
) -> reqwest::Result<Result<DynamicImage, image::ImageError>> {
    let mut buf = Vec::new();
    let mut resp = client.get(url).send()?;
    std::io::copy(&mut resp, &mut buf).unwrap();
    Ok(image::load_from_memory(&buf))
}

const PRINTER_DOTS_PER_LINE: u32 = 384;
fn print_image<W: std::io::Write>(
    printer: &mut Printer<W>,
    source_image: DynamicImage,
) -> Result<(), &'static str> {
    // Size image to fit the printer
    let resized = source_image
        .resize(PRINTER_DOTS_PER_LINE, 9000, FilterType::Triangle)
        .to_rgb();

    // Convert the image to a ditherable format
    let ditherable_image = dither::prelude::Img::<RGB<u8>>::new(
        resized.pixels().map(|p| RGB::from(p.data)),
        resized.width(),
    )
    .ok_or("Failed to convert to ditherable image")?
    .convert_with(|rgb| rgb.convert_with(f64::from));

    // Dither the image using the SIERRA 3 algorithm
    let dithering_palette = [RGB::from((255, 255, 255)), RGB::from((0, 0, 0))];
    let dithered = SIERRA_3
        .dither(ditherable_image, palette::quantize(&dithering_palette))
        .convert_with(|rgb| rgb.convert_with(dither::prelude::clamp_f64_to_u8));

    // Switch the image back to a consumable format for the printer
    let converted_dither =
        ImageBuffer::from_raw(dithered.width(), dithered.height(), dithered.raw_buf())
            .ok_or("Failed to convert dithered image to buffer")?;
    let printer_image = escposify::img::Image::from(DynamicImage::ImageRgb8(converted_dither));

    // Print the image
    let printer_err_msg = "Failed to write to printer";
    printer.align("lt").map_err(|_| printer_err_msg)?;
    printer
        .bit_image(&printer_image, None)
        .map_err(|_| printer_err_msg)?;
    Ok(())
}
