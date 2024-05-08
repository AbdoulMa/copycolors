use crate::ColorTrait;
use crate::ImageTrait;
use color_thief::{Color, ColorFormat};
use image::{self, DynamicImage, ImageResult};
use std::{cmp::Ordering, path::Path};

#[derive(Debug)]
pub struct ImageFile {
    file_path: String,
    // TODO: Result, to manage problematic image at read
    // and more important to not disturb directory case
    pub image: ImageResult<DynamicImage>,
}

impl ImageFile {
    pub fn new(file_path: String) -> Self {
        let mut image_file = Self {
            file_path,
            image: Ok(DynamicImage::new_rgb8(0, 0)),
        };
        image_file.set_image_from_file_path();
        image_file
    }

    pub fn set_image_from_file_path(&mut self) {
        self.image = image::open(Path::new(&self.file_path));
    }

    pub fn get_colors_from_images(
        &self,
        nb_colors: u8,
        excluded_colors: Vec<Color>,
        bc_color: Option<Color>,
    ) -> Vec<Color> {
        let fv = self
            .image
            .as_ref()
            .unwrap()
            .filtered_image_bytes(&excluded_colors);
        let (color_bytes, color_format) = if !excluded_colors.is_empty() {
            (fv.as_slice(), ColorFormat::Rgb)
        } else {
            (
                self.image.as_ref().unwrap().as_bytes(),
                self.image.as_ref().unwrap().color_format(),
            )
        };
        let mut colors =
            color_thief::get_palette(color_bytes, color_format, 10, nb_colors as u8).unwrap();
        if let Some(cc) = bc_color {
            colors.sort_by(|c1, c2| {
                c1.contrast_with(cc)
                    .partial_cmp(&c2.contrast_with(cc))
                    .map(Ordering::reverse)
                    .unwrap()
            });
        }
        colors
    }
}
