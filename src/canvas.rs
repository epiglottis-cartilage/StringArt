use glam::Vec2;
use image::GenericImageView;

#[derive(Debug, Clone)]
pub struct Canvas {
    buf: image::ImageBuffer<image::Luma<f32>, Vec<f32>>,
}
impl Into<image::DynamicImage> for Canvas {
    fn into(self) -> image::DynamicImage {
        image::DynamicImage::from(self.buf.clone())
    }
}
impl Canvas {
    pub fn new(img_size: u32, fill: f32) -> Self {
        let img_buf = image::ImageBuffer::from_fn(img_size, img_size, |_, _| image::Luma([fill]));
        Self { buf: img_buf }
    }
    pub fn from(image: &image::DynamicImage, img_size: u32) -> Self {
        let (w, h) = image.dimensions();
        let size = w.min(h);
        let img_buf = image
            .crop_imm((w - size) / 2, (h - size) / 2, size, size)
            .resize(img_size, img_size, image::imageops::FilterType::Lanczos3)
            .grayscale()
            .to_luma32f();
        Self { buf: img_buf }
    }
    pub fn invert(&mut self) {
        for pixel in self.buf.pixels_mut() {
            pixel.0[0] = 1.0 - pixel.0[0];
        }
    }
    pub fn get_pixel(&self, coord: glam::Vec2) -> f32 {
        self.buf.get_pixel(coord.x as _, coord.y as _)[0]
    }
    pub fn get_pixel_mut(&mut self, coord: glam::Vec2) -> &mut f32 {
        &mut self.buf.get_pixel_mut(coord.x as _, coord.y as _)[0]
    }
    pub fn line_space_coord(&self, start: Vec2, end: Vec2) -> Vec<Vec2> {
        let step = (end - start).normalize();
        let steps = ((end - start).length() / step.length()).ceil() as u32;
        (0..steps).map(|i| start + step * i as f32).collect()
    }
}
