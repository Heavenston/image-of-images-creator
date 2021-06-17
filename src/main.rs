mod image_dictionary;
mod image_of_image_er;

use clap::{Arg, App};
use std::convert::{TryFrom};
use std::str::FromStr;
use colored::*;
use rayon::prelude::*;
use image::GenericImageView;

fn try_from_validator<T: TryFrom<String>>(error: String) -> impl Fn(String) -> Result<(), String> {
    move |path| match T::try_from(path) {
        Ok(..) => Ok(()),
        Err(..) => Err(error.clone())
    }
}
fn from_string_validator<T: FromStr>(error: String) -> impl Fn(String) -> Result<(), String> {
    move |path| match path.parse::<T>() {
        Ok(..) => Ok(()),
        Err(..) => Err(error.clone())
    }
}
fn try_parse<T: FromStr>(text: Option<&str>) -> Option<T> {
    text.map(|a| a.parse().ok()).flatten()
}

fn main() {
    let matches = App::new("Image of image creator")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("TARGET_IMAGE")
            .help("Path to the target image")
            .validator(try_from_validator::<std::path::PathBuf>("Invalid path".into()))
            .required(true)
            .index(1)
        )
        .arg(Arg::with_name("DICTIONARY")
            .help("Path to the dictionary folder")
            .validator(try_from_validator::<std::path::PathBuf>("Invalid path".into()))
            .required(true)
            .index(2)
        )
        .arg(Arg::with_name("WIDTH")
            .help("Resize the target image to this width before processing")
            .required(false)
            .short("w")
            .long("width")
            .takes_value(true)
            .validator(from_string_validator::<u32>("Invalid path".into()))
        )
        .arg(Arg::with_name("HEIGHT")
            .help("Resize the target image to this height before processing")
            .required(false)
            .takes_value(true)
            .short("h")
            .long("height")
            .validator(from_string_validator::<u32>("Invalid path".into()))
        )
        .get_matches();

    let mut target_image = match image::io::Reader::open(
        matches.value_of("TARGET_IMAGE").unwrap()
    ) {
        Ok(i) => match i.decode() {
            Ok(i) => i,
            Err(..) => {
                println!("{}", "Invalid image".red());
                return
            }
        },
        Err(..) => {
            println!("{}", "Could not read target image".red());
            return
        }
    };
    let mut height = 0;
    let mut width = 0;
    match (try_parse::<u32>(matches.value_of("WIDTH")),
           try_parse::<u32>(matches.value_of("HEIGHT"))) {
        (Some(w), Some(h)) => {
            height = h;
            width = w;
        }
        (Some(w), None) => {
            height = w * (target_image.height() / target_image.width());
            width = w;
        }
        (None, Some(h)) => {
            width = h * (target_image.width() / target_image.height());
            height = h;
        }
        _ => (),
    }
    if height != 0 && width != 0 {
        target_image = target_image.resize_exact(width, height, image::imageops::Gaussian);
    }
    println!("Loaded image is {}x{}", target_image.width(), target_image.height());

    let dict_reader = match image_dictionary::ImageDictionaryReader::open(
        matches.value_of("DICTIONARY").unwrap(),
        (12, 12)
    ) {
        Ok(d) => d,
        Err(e) => { println!("{}", e.red()); return },
    };
    println!("Loading {} images", dict_reader.len());
    let mut splits = dict_reader.split(10);
    splits.par_iter_mut()
        .for_each(|split| {
            while split.process_image().unwrap_or(true) {}
            println!("Done one")
        });
    let image_dictionary = dict_reader.build_split(splits);
    println!("Processing...");

    let new_image = image_of_image_er::image_of_image(&image_dictionary, &target_image.to_rgb8());
    println!("Final image size: {}x{}", new_image.width(), new_image.height());
    println!("Saving...");
    new_image.save("hello.png").unwrap();
}
