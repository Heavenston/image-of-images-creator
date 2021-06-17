mod image_dictionary;

use clap::{Arg, App};
use std::convert::{TryFrom};
use std::str::FromStr;
use colored::*;
use rayon::prelude::*;

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
            .validator(from_string_validator::<u32>("Invalid path".into()))
        )
        .get_matches();

    let target_image = match image::io::Reader::open(
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
    }.to_rgba8();

    let dict_reader = match image_dictionary::ImageDictionaryReader::open(
        matches.value_of("DICTIONARY").unwrap()
    ) {
        Ok(d) => d,
        Err(e) => { println!("{}", e.red()); return },
    };
    println!("Loading {} images", dict_reader.len());
    let mut splits = dict_reader.split(10);
    splits.par_iter_mut()
        .for_each(|split| {
            while split.process_image().unwrap() {}
        });
    let image_dictionary = dict_reader.build_split(splits);
    image_dictionary.save().unwrap();
}
