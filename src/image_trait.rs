use crate::{Color, ColorFormat, ColorTrait};
use image::DynamicImage;

const MAX_DISTANCE: f32 = 585225.0;

pub trait ImageTrait {
    fn color_format(&self) -> Result<ColorFormat, String>;
    fn color_size(&self) -> usize;
    fn filtered_image_bytes(&self, exc_color: &[Color]) -> Result<Vec<u8>, String>;
}

impl ImageTrait for DynamicImage {
    fn color_format(&self) -> Result<ColorFormat, String> {
        match self.color() {
            image::ColorType::Rgb8 => {
                return Ok(ColorFormat::Rgb);
            }
            image::ColorType::Rgba8 => {
                return Ok(ColorFormat::Rgba);
            }
            other => {
                return Err(format!(
                    "Sorry, images with {other:?} color type pixels are not supported."
                ));
            }
        }
    }

    fn color_size(&self) -> usize {
        match self.color_format() {
            Ok(ColorFormat::Rgb) | Ok(ColorFormat::Bgr) => 3,
            _ => 4,
        }
    }

    fn filtered_image_bytes(&self, excluded_colors: &[Color]) -> Result<Vec<u8>, String> {
        let color_format = self.color_format();
        if color_format.is_err() {
            return Err(color_format.err().unwrap());
        }
        let color_size = self.color_size();
        let pixels = self.as_bytes();
        let mut colors_bytes_vec = vec![];
        for i in (0..pixels.len()).step_by(color_size) {
            let pixel_color =
                Color::pixels_to_rbg(&pixels[i..(i + color_size)], self.color_format().unwrap());
            let mut keep_pixel = true;
            for ex_color in excluded_colors {
                let pixel_dist = pixel_color.delta_rgb(Color {
                    r: ex_color.r,
                    g: ex_color.g,
                    b: ex_color.b,
                });
                let pixel_dist_pct = pixel_dist / MAX_DISTANCE;
                if pixel_dist_pct < 0.05 {
                    // threshold 5%
                    keep_pixel = false;
                }
            }
            if keep_pixel {
                colors_bytes_vec.extend(pixel_color.to_slice());
            }
        }
        Ok(colors_bytes_vec)
    }
}
