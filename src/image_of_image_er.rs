use crate::image_dictionary::{ImageDictionary, DictionaryColor};
use image::{DynamicImage, GenericImageView, Pixel, GenericImage, ImageBuffer, RgbImage, Rgb};
use palette::{Srgb, Hsl};

pub fn image_of_image(dictionary: &ImageDictionary, target_image: &RgbImage) -> RgbImage {
    let mut new_image: RgbImage = RgbImage::new(target_image.width() * dictionary.images_size.0, target_image.height() * dictionary.images_size.1);

    target_image.enumerate_pixels()
        .for_each(|(x, y, p)| {
            let p = target_image.get_pixel(x, y);
            let (_, image) = dictionary.get_closest(&DictionaryColor::from(Srgb::new(
                p.0[0] as f32 / 255.,
                p.0[1] as f32 / 255.,
                p.0[2] as f32 / 255.,
            )));
            image.enumerate_pixels().for_each(|(nx, ny, p)| {
                let tp = new_image.get_pixel_mut(nx + x * dictionary.images_size.0,
                                        ny + y * dictionary.images_size.1);
                *tp = *p;
            });
        });

    new_image
}