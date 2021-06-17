use std::fs;
use palette::{Srgb, LinSrgb,  Mix};
use std::path::PathBuf;
use image::{GenericImageView,};
use std::sync::Mutex;
use std::ops::Deref;

pub type ColorComponent = f32;

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct ImageDictionaryCacheFile<A, B>
where A: Deref<Target = [PathBuf]>, B: Deref<Target = [Srgb<ColorComponent>]> {
    images: A,
    colors: B,
}

pub struct ImageDictionaryReader {
    dictionary_path: PathBuf,
    remaining_read_images: Vec<PathBuf>,

    images: Mutex<Vec<PathBuf>>,
    colors: Mutex<Vec<Srgb<ColorComponent>>>,
}
impl ImageDictionaryReader {
    pub fn open(folder: &str) -> Result<ImageDictionaryReader, String> {
        let dictionary_path = std::path::PathBuf::from(folder);
        match dictionary_path.metadata() {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir_all(&dictionary_path).map_err(|_| "Could not create folder".to_string())?;
            }
            Err(_) =>
                return Err("Unknown error".into()),
            Ok(d) if d.is_file() => return Err("Dictionary can't be a file".into()),
            _ => ()
        };
        let save_path = dictionary_path.join("dictionary_cache.json");

        let (images, colors) = if save_path.exists() {
            let mut file = fs::File::open(&save_path).map_err(|_| "Could not read dictionary".to_string())?;
            let ImageDictionaryCacheFile::<Vec<_>, Vec<_>> { images, colors } = serde_json::from_reader(&mut file).map_err(|_| "Invalid dictionary".to_string())?;
            (images, colors)
        }
        else {
            Default::default()
        };

        let remaining_read_images = fs::read_dir(&dictionary_path)
            .map_err(|_| "Could not read directory".to_string())?
            .into_iter()
            .filter_map(|a| a.ok())
            .filter(|entry| entry.metadata().map(|m| m.is_file()).unwrap_or(false))
            .map(|entry| entry.path())
            .filter(|path| path.file_name()
                .map(|a| a.to_str()).flatten()
                != Some("dictionary_cache.json"))
            .filter(|p| !images.contains(&p.strip_prefix(&dictionary_path).unwrap().to_path_buf()))
            .collect::<Vec<_>>();

        Ok(ImageDictionaryReader {
            images: Mutex::new(images.iter().map(|p| dictionary_path.join(p)).collect()), colors: Mutex::new(colors),
            dictionary_path,
            remaining_read_images
        })
    }

    pub fn len(&self) -> usize {
        self.remaining_read_images.len() + self.images.lock().unwrap().len()
    }

    pub fn split(&self, chunk_size: usize) -> Vec<ImageDictionaryReaderChunk> {
        let origin = &*self;
        self.remaining_read_images
            .chunks(chunk_size)
            .map(|c|
                ImageDictionaryReaderChunk {
                    origin,
                    remaining_read_images: c,

                    images: Vec::new(),
                    colors: Vec::new(),
                }
            )
            .collect()
    }

    pub fn build_split<'a>(&'a self, chunks: Vec<ImageDictionaryReaderChunk<'a>>) -> ImageDictionary {
        let mut colors = Vec::new();
        std::mem::swap(&mut *self.colors.lock().unwrap(), &mut colors);
        let mut images = Vec::new();
        std::mem::swap(&mut *self.images.lock().unwrap(), &mut images);

        let mut dict = ImageDictionary {
            dictionary_path: self.dictionary_path.clone(),
            colors,
            images,
        };
        chunks.into_iter().for_each(|mut d| {
            assert_eq!(self as *const Self, d.origin as *const Self);
            dict.colors.append(&mut d.colors);
            dict.images.append(&mut d.images);
        });

        dict
    }
}

pub struct ImageDictionaryReaderChunk<'a> {
    origin: &'a ImageDictionaryReader,
    remaining_read_images: &'a [PathBuf],

    images: Vec<PathBuf>,
    colors: Vec<Srgb<ColorComponent>>,
}

impl<'a> ImageDictionaryReaderChunk<'a> {
    /// Return the amount of images remaining to be processed
    pub fn len(&self) -> usize {
        self.remaining_read_images.len()
    }

    pub fn process_image(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        if self.remaining_read_images.len() == 0 { return Ok(false) }
        let path = self.remaining_read_images[0].clone();
        self.remaining_read_images = &self.remaining_read_images[1..];
        let image = image::io::Reader::open(&path)?.decode()?;
        let color = image
            .pixels()
            .map(|(.., color)| color)
            .map(|color| (
                color.0[0] as f32 / 255.,
                color.0[1] as f32 / 255.,
                color.0[2] as f32 / 255.,
            ))
            .map(
                |color| Srgb::from_components(color).into_linear()
            )
            .fold(
                LinSrgb::default(),
                |a, b| a.mix(&b, 0.5)
            );
        self.images.push(path);
        self.colors.push(Srgb::from_linear(color));
        Ok(true)
    }
}

#[derive(Default)]
pub struct ImageDictionary {
    dictionary_path: PathBuf,
    images: Vec<PathBuf>,
    colors: Vec<Srgb<ColorComponent>>,
}
impl ImageDictionary {
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = std::fs::File::create(self.dictionary_path.join("dictionary_cache.json"))?;
        serde_json::to_writer_pretty(&mut file, &ImageDictionaryCacheFile {
            images: self.images.iter().map(|a| {
                a.strip_prefix(&self.dictionary_path).unwrap().to_path_buf()
            }).collect::<Vec<_>>(),
            colors: self.colors.as_slice(),
        })?;
        Ok(())
    }
}
