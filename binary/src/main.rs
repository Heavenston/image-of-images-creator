use clap::{Arg, App};
use std::convert::{TryFrom};
use std::str::FromStr;
use colored::*;
use rayon::prelude::*;
use image::GenericImageView;
use image_of_images_creator::*;

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
        .arg(Arg::with_name("OUTPUT")
            .help("Path to the output file")
            .validator(try_from_validator::<std::path::PathBuf>("Invalid path".into()))
            .required(false)
            .default_value("output.png")
            .index(3)
        )
        .arg(Arg::with_name("WIDTH")
            .help("Resize the target image to this width before processing")
            .required(false)
            .short("w")
            .long("width")
            .takes_value(true)
            .validator(from_string_validator::<u32>("Invalid number".into()))
        )
        .arg(Arg::with_name("HEIGHT")
            .help("Resize the target image to this height before processing")
            .required(false)
            .takes_value(true)
            .short("h")
            .long("height")
            .validator(from_string_validator::<u32>("Invalid number".into()))
        )
        .arg(Arg::with_name("PIX_WIDTH")
            .help("Resize dictionary images before processing")
            .required(false)
            .takes_value(true)
            .short("pw")
            .long("pixel_width")
            .default_value("32")
            .validator(from_string_validator::<u32>("Invalid number".into()))
        )
        .get_matches();

    let pixel_width = matches.value_of("PIX_WIDTH").unwrap().parse::<u32>().unwrap();

    println!("Loading dictionary...");
    let dict_reader = match ImageDictionaryReader::open(
        matches.value_of("DICTIONARY").unwrap(),
        (pixel_width, pixel_width)
    ) {
        Ok(d) => d,
        Err(e) => { println!("{}", e.red()); return },
    };
    println!("Loading {} images", dict_reader.len());
    let mut splits = dict_reader.split(10);
    splits.par_iter_mut()
        .for_each(|split| {
            while split.process_image().unwrap_or(true) {}
        });
    let image_dictionary = dict_reader.build_split(splits);

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
            height = (w as f32 * (target_image.height() as f32 / target_image.width() as f32)) as u32;
            width = w;
        }
        (None, Some(h)) => {
            width = (h as f32 * (target_image.width() as f32 / target_image.height() as f32)) as u32;
            height = h;
        }
        _ => (),
    }
    if height != 0 && width != 0 {
        target_image = target_image.resize_exact(width, height, image::imageops::Gaussian);
    }
    println!("Loaded image is {}x{}", target_image.width(), target_image.height());

    println!("Processing...");
    let new_image = image_of_image(&image_dictionary, &target_image.to_rgb8());
    println!("Final image size: {}x{}", new_image.width(), new_image.height());
    println!("Saving...");
    new_image.save(matches.value_of("OUTPUT").unwrap()).unwrap();
}
