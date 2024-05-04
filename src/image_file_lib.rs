use image::{self, DynamicImage, Rgb};

use std::{cmp::Ordering, io, path::Path, process};

use crate::ColorTrait;
use crate::ImageTrait;

use color_thief::{Color, ColorFormat};

#[derive(Debug, Clone)]
pub struct ImageFile {
    file_path: String,
    // TODO: Result, to manage problematic image at read
    // and more important to not disturb directory case
    image: DynamicImage,
}

impl ImageFile {
    pub fn new(file_path: String) -> Self {
        let mut image_file = Self {
            file_path,
            image: DynamicImage::new_rgb8(0, 0),
        };
        image_file.set_image_from_file_path();
        image_file
    }

    fn set_file_path(&mut self, file_path: String) {
        self.file_path = file_path
    }
    pub fn set_image_from_file_path(&mut self) {
        self.image = image::open(Path::new(&self.file_path)).unwrap_or_else(|err| match err {
            image::ImageError::IoError(io_error) => match io_error.kind() {
                io::ErrorKind::NotFound => {
                    eprintln!("File not found.\nPlease be sure you provide the correct path!");
                    process::exit(1);
                }
                _ => {
                    eprintln!("Error while opening the file!");
                    process::exit(1);
                }
            },
            _ => {
                eprintln!("Error while opening the file!");
                process::exit(1);
            }
        });
    }

    pub fn get_colors_from_images(
        &self,
        nb_colors: u8,
        excluded_colors: Vec<Color>,
        bc_color: Option<Color>,
    ) -> Vec<Color> {
        let fv = self.image.filtered_image_bytes(&excluded_colors);
        let (color_bytes, color_format) = if !excluded_colors.is_empty() {
            (fv.as_slice(), ColorFormat::Rgb)
        } else {
            (self.image.as_bytes(), self.image.color_format())
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
