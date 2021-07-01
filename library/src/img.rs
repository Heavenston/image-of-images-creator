use image::{ RgbImage, GenericImage };
use palette::Srgb;
use std::path::Path;

pub trait Img: Sized {
    type Pixel: Into<Srgb<u8>>;

    fn new(width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>>;
    fn read(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>>;

    fn img_size(&self) -> [u32; 2];
    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel;
    fn set_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel);

    fn absorbe(&mut self, other: &Self, x: u32, y: u32);
    fn resize(&self, width: u32, height: u32) -> Self;

    fn for_each_pixels(&self, callback: impl FnMut(u32, u32, Self::Pixel) -> ());
}

impl Img for RgbImage {
    type Pixel = Srgb<u8>;

    fn new(width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(RgbImage::new(width, height))
    }
    fn read(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(image::io::Reader::open(path)?.decode()?.into_rgb8())
    }

    fn img_size(&self) -> [u32; 2] {
        [self.width(), self.height()]
    }
    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        let p = self.get_pixel(x, y).0;
        Srgb::new(p[0], p[1], p[2])
    }
    fn set_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        let (r, g, b) = pixel.into_components();
        self.put_pixel(x, y, image::Rgb([r, g, b]));
    }

    fn absorbe(&mut self, other: &Self, x: u32, y: u32) {
        self.copy_from(other, x, y).unwrap();
    }
    fn resize(&self, width: u32, height: u32) -> Self {
        image::imageops::resize(self, width, height, image::imageops::FilterType::Gaussian)
    }
    
    fn for_each_pixels(&self, mut callback: impl FnMut(u32, u32, Self::Pixel) -> ()) {
        self.enumerate_pixels().for_each(|(x, y, c)| {
            let p = c.0;
            (callback)(x, y, Srgb::new(p[0], p[1], p[2]))
        });
    }
}

#[cfg(ocv)]
mod open_impl {
    use super::*;
    use opencv::prelude::*;
    use opencv::core::{ Vec3, CV_8UC3, Rect };
    
    impl Img for Mat {
        type Pixel = Srgb<u8>;

        fn new(width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(unsafe { Mat::new_rows_cols(width as i32, height as i32, CV_8UC3)? })
        }
        fn read(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(opencv::imgcodecs::imread(&path.as_ref().to_string_lossy(), opencv::imgcodecs::ImreadModes::IMREAD_COLOR as i32)?)
        }

        fn img_size(&self) -> [u32; 2] {
            [self.cols() as u32, self.rows() as u32]
        }
        fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
            let p = self.at_2d::<Vec3<u8>>(x as i32, y as i32).unwrap();
            Self::Pixel::new(p[2], p[1], p[0])
        }
        fn set_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
            let p = self.at_2d_mut::<Vec3<u8>>(x as i32, y as i32).unwrap();
            p[0] = pixel.blue;
            p[1] = pixel.green;
            p[2] = pixel.red;
        }

        fn absorbe(&mut self, other: &Self, x: u32, y: u32) {
            other.copy_to(&mut Mat::roi(self, Rect::new(x as i32, y as i32, other.cols(), other.rows())).unwrap()).unwrap();
        }
        fn resize(&self, width: u32, height: u32) -> Self {
            let mut dst = Self::new(width, height).unwrap();
            let size = self.size().unwrap();
            opencv::imgproc::resize(self, &mut dst, size, 0., 0., 0).unwrap();
            dst
        }
        fn for_each_pixels(&self, mut callback: impl FnMut(u32, u32, Self::Pixel) -> ()) {
            for x in 0..self.cols() as u32 {
                for y in 0..self.rows() as u32 {
                    (callback)(x, y, self.get_pixel(x, y));
                }
            }
        }
    }
}










