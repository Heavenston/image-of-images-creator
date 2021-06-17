use crate::image_dictionary::{ImageDictionary, DictionaryColor};
use image::{DynamicImage, GenericImageView, Pixel, GenericImage, ImageBuffer, RgbImage, Rgb};
use palette::{Srgb, Hsl};

pub fn image_of_image(dictionary: &ImageDictionary, target_image: &DynamicImage) -> RgbImage {
    let mut new_image = RgbImage::new(target_image.width(), target_image.height());

    new_image.enumerate_pixels_mut()
        .for_each(|(x, y, pixel): (_, _, &mut Rgb<u8>)| {
            let p = target_image.get_pixel(x, y);
            let closest = Srgb::from(*dictionary.get_closest(&DictionaryColor::from(Srgb::new(
                p.0[0] as f32 / 255.,
                p.0[1] as f32 / 255.,
                p.0[2] as f32 / 255.,
            ))));
            pixel.0[0] = (closest.red * 255.) as u8;
            pixel.0[1] = (closest.green * 255.) as u8;
            pixel.0[2] = (closest.blue * 255.) as u8;
        });

    new_image
}