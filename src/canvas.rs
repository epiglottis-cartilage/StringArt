use image::GenericImageView;

use crate::coord::Coord;

pub type GrayImage = image::ImageBuffer<image::Luma<u16>, Vec<u16>>;
#[derive(Debug, Clone)]
pub struct Canvas(pub image::ImageBuffer<image::Luma<f32>, Vec<f32>>);
impl Canvas {
    pub fn invert(&mut self) {
        for pixel in self.0.pixels_mut() {
            pixel.0[0] = 1.0 - pixel.0[0];
        }
    }
    pub fn save_to_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> image::error::ImageResult<()> {
        self.to_image().save(path)
    }

    pub fn from_image(image: &image::DynamicImage, img_size: u32) -> Canvas {
        let (w, h) = image.dimensions();
        let size = w.min(h);
        let image: GrayImage = image
            .crop_imm((w - size) / 2, (h - size) / 2, size, size)
            .resize(img_size, img_size, image::imageops::FilterType::Lanczos3)
            .grayscale()
            .into();
        Canvas(
            image::ImageBuffer::from_vec(
                img_size,
                img_size,
                image
                    .pixels()
                    .map(|pixel| pixel[0] as f32 / std::u16::MAX as f32)
                    .collect(),
            )
            .unwrap(),
        )
    }
    pub fn to_image(&self) -> GrayImage {
        GrayImage::from_vec(
            self.0.width(),
            self.0.height(),
            self.0
                .pixels()
                .map(|pixel| (pixel[0] * std::u16::MAX as f32) as _)
                .collect(),
        )
        .unwrap()
    }
    pub fn get_pixel(&self, coord: Coord) -> f32 {
        self.0.get_pixel(coord.x as _, coord.y as _)[0]
    }
    pub fn get_pixel_mut(&mut self, coord: Coord) -> &mut f32 {
        &mut self.0.get_pixel_mut(coord.x as _, coord.y as _)[0]
    }
    pub fn overlay(&mut self, other: &Canvas) {
        self.0
            .pixels_mut()
            .zip(other.0.pixels())
            .for_each(|(s, e)| s[0] += e[0]);
    }
}
