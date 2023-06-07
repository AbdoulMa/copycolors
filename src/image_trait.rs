use image::DynamicImage;
use crate::{Color, ColorFormat, ColorTrait};

const MAX_DISTANCE: f32 = 585225.0;

pub trait ImageTrait {
    fn color_format(&self) -> ColorFormat;
    fn color_size(&self) -> usize;
    fn filtered_image_bytes(&self, exc_color: &[Color]) -> Vec<u8>;
}

impl ImageTrait for DynamicImage {
    fn color_format(&self) -> ColorFormat {
        match self.color() {
            image::ColorType::Rgb8 => ColorFormat::Rgb,
            image::ColorType::Rgba8 => ColorFormat::Rgba,
            _ => unreachable!(),
        }
    }

    fn color_size(&self) -> usize {
        match self.color_format() {
            ColorFormat::Rgb | ColorFormat::Bgr => 3,
            _ => 4,
        }
    }

    fn filtered_image_bytes(&self, excluded_colors: &[Color]) -> Vec<u8> {
        let color_size = self.color_size();
        let pixels = self.as_bytes();
        let mut colors_bytes_vec = vec![];
        for i in (0..pixels.len()).step_by(color_size) {
            let pixel_color =
                Color::pixels_to_rbg(&pixels[i..(i + color_size)], self.color_format());
            let mut keep_pixel = true;
            for ex_color in excluded_colors {
                let pixel_dist = pixel_color.delta_rgb(Color {
                    r: ex_color.r,
                    g: ex_color.g,
                    b: ex_color.b,
                });
                let pixel_dist_pct = pixel_dist / MAX_DISTANCE;
                if pixel_dist_pct < 0.075 {
                    // threshold 7.5%
                    keep_pixel = false;
                }
            }
            if keep_pixel {
                colors_bytes_vec.extend(pixel_color.to_slice());
            }
        }
        colors_bytes_vec
    }
}