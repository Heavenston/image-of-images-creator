use std::fs;
use palette::{Srgb, Lab};
use std::path::PathBuf;
use image::{GenericImageView, RgbImage};
use std::sync::Mutex;

pub type ColorComponent = f32;
pub type DictionaryColor = Lab<palette::white_point::D65, ColorComponent>;

pub struct ImageDictionaryReader {
    remaining_read_images: Vec<PathBuf>,
    images_size: (u32, u32),

    images: Mutex<Vec<RgbImage>>,
    colors: Mutex<Vec<DictionaryColor>>,
}
impl ImageDictionaryReader {
    pub fn open(folder: &str, images_size: (u32, u32)) -> Result<ImageDictionaryReader, String> {
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
        let remaining_read_images = fs::read_dir(&dictionary_path)
            .map_err(|_| "Could not read directory".to_string())?
            .into_iter()
            .filter_map(|a| a.ok())
            .filter(|entry| entry.metadata().map(|m| m.is_file()).unwrap_or(false))
            .map(|entry| entry.path())
            .collect::<Vec<_>>();

        Ok(ImageDictionaryReader {
            images: Mutex::new(Vec::default()), colors: Mutex::new(Vec::default()),
            images_size,
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

                    images_size: self.images_size,
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
            colors,
            images,
            images_size: self.images_size,
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
    images_size: (u32, u32),

    images: Vec<RgbImage>,
    colors: Vec<DictionaryColor>,
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
        let normal_image = image::io::Reader::open(path)?.decode()?;
        let amount_of_pixels = (normal_image.height() * normal_image.width()) as f32;
        let color = normal_image.pixels()
            .map(|(.., pixel)| [pixel.0[0] as f32 / 255., pixel.0[1] as f32 / 255., pixel.0[2] as f32 / 255.])
            .map(|c| Srgb::new(c[0], c[1], c[2]))
            .fold(Srgb::new(0., 0., 0.), |mut mean, color| {
                mean.red += color.red / amount_of_pixels;
                mean.green += color.green / amount_of_pixels;
                mean.blue += color.blue / amount_of_pixels;
                mean
            });
        let image = normal_image.resize_exact(self.images_size.0,self.images_size.1, image::imageops::Gaussian).to_rgb8();
        self.images.push(image);
        self.colors.push(DictionaryColor::from(color));
        Ok(true)
    }
}

#[derive(Default)]
pub struct ImageDictionary {
    images: Vec<RgbImage>,
    colors: Vec<DictionaryColor>,
    pub images_size: (u32, u32),
}
impl ImageDictionary {
    pub fn get_closest(&self, t_color: &DictionaryColor) -> (&DictionaryColor, &RgbImage) {
        let best_index = self.colors
            .iter()
            .enumerate()
            .fold(
                (10000., 0usize),
                |(best_score, best_color), (i, color)| {
                    let score = (t_color.l - color.l).powi(2) + (t_color.a - color.a).powi(2) + (t_color.b - color.b).powi(2);
                    if score < best_score {
                        (score, i)
                    }
                    else {
                        (best_score, best_color)
                    }
                }
            ).1;

        (&self.colors[best_index], &self.images[best_index])
    }
}
