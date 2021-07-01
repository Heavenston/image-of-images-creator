use crate::image_dictionary::{ImageDictionary, DictionaryColor};
use crate::Img;

pub fn image_of_image<I: Img>(dictionary: &ImageDictionary<I>, target_image: &I) -> I {
    let [width, height] = target_image.img_size();
    let mut new_image = I::new(width * dictionary.images_size.0, height * dictionary.images_size.1).unwrap();
    let image_ptr = &mut new_image as *mut _ as usize;


    target_image.for_each_pixels(|x, y, p| {
        let new_image = unsafe { &mut *(image_ptr as *mut I) };
        let (_, image) = dictionary.get_closest(&DictionaryColor::from(p.into().into_format::<f32>()));
        new_image.absorbe(&image, x * dictionary.images_size.0, y * dictionary.images_size.1);
    });

    new_image
}
