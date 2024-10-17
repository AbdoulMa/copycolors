use crate::process;
use crate::{Color, ColorFormat};
use crossterm::style::Color as TerminalColor;
use regex::Regex;
use std::borrow::Borrow; // Vector borrowing
use std::error;

pub trait ColorTrait {
    fn hexadecimal_str(&self) -> String;
    fn rgb_str(&self) -> String;
    fn color_brightness(&self) -> f32;
    fn best_contrast<'a, T: Borrow<Color>>(&'a self, colors: &'a [T]) -> &'a T;
    fn contrast_with(&self, col: Color) -> f32;
    fn to_slice(&self) -> [u8; 3];
    fn hex_to_rgb(hex_code: &str) -> Result<Color, Box<dyn error::Error>>;
    fn pixels_to_rbg(arr: &[u8], color_format: ColorFormat) -> Color;
    fn delta_rgb(&self, col2: Color) -> f32;
    fn to_term_color(&self) -> TerminalColor;
}

impl ColorTrait for Color {
    fn hexadecimal_str(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    fn rgb_str(&self) -> String {
        format!("RGB({},{},{})", self.r, self.g, self.b)
    }

    fn color_brightness(&self) -> f32 {
        ((299 * (self.r as u32) + 587 * (self.g as u32) + 114 * (self.b as u32)) as f32) / 1000.0
    }

    // Borrow String Vector
    //    https://stackoverflow.com/questions/73839107/how-to-write-a-function-that-accepts-a-vec-of-borrowed-or-owned-elements?rq=1
    fn best_contrast<'a, T: Borrow<Color>>(&self, text_colors: &'a [T]) -> &'a T {
        // https://www.had2know.org/technology/color-contrast-calculator-web-design.html
        assert!(text_colors.len() > 1);
        // bg brightness
        let bg_bgt = self.color_brightness();
        // text color brightness
        let mut res = &text_colors[0];
        let text_bgt = res.borrow().color_brightness();
        let bgt_diff = (bg_bgt - text_bgt).abs();
        for bc in &text_colors[1..] {
            let new_bgt = bc.borrow().color_brightness();
            let new_bgt_diff = (bg_bgt - new_bgt).abs();
            if new_bgt_diff > bgt_diff {
                res = bc;
            }
        }
        res
    }

    fn contrast_with(&self, col: Color) -> f32 {
        (self.color_brightness() - col.color_brightness()).abs()
    }

    fn to_slice(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    fn hex_to_rgb(hex_code: &str) -> Result<Color, Box<dyn error::Error>> {
        let hex_regex = Regex::new("^(?i)#[\\da-f]{6}$").unwrap();
        let hex_code = hex_code.trim();
        if !hex_regex.is_match(hex_code) {
            eprintln !("{hex_code} is not a valid hexadecimal code.\nPlease provide a valid hex code, and try again!");
            process::exit(1);
        }
        let r = u8::from_str_radix(&hex_code[1..3], 16).unwrap();
        let g = u8::from_str_radix(&hex_code[3..5], 16).unwrap();
        let b = u8::from_str_radix(&hex_code[5..7], 16).unwrap();
        Ok(Color { r, g, b })
    }

    fn pixels_to_rbg(arr: &[u8], color_format: ColorFormat) -> Color {
        match color_format {
            ColorFormat::Rgb => Color {
                r: arr[0],
                g: arr[1],
                b: arr[2],
            },
            ColorFormat::Rgba => Color {
                r: arr[0],
                g: arr[1],
                b: arr[2],
            },
            ColorFormat::Argb => Color {
                r: arr[1],
                g: arr[2],
                b: arr[3],
            },
            ColorFormat::Bgr => Color {
                r: arr[2],
                g: arr[1],
                b: arr[0],
            },
            ColorFormat::Bgra => Color {
                r: arr[2],
                g: arr[1],
                b: arr[0],
            },
        }
    }

    // https://gist.github.com/ryancat/9972419b2a78f329ce3aebb7f1a09152
    fn delta_rgb(&self, col2: Color) -> f32 {
        let drp2 = (self.r as f32 - col2.r as f32).powf(2.0) as f32;
        let dgp2 = (self.g as f32 - col2.g as f32).powf(2.0) as f32;
        let dbp2 = (self.b as f32 - col2.b as f32).powf(2.0) as f32;
        let t = (self.r as f32 + col2.r as f32) as f32 / 2.0;

        2.0 * drp2 + 4.0 * dgp2 + 3.0 * dbp2 + t * (drp2 - dbp2) / 256.0
    }

    fn to_term_color(&self) -> TerminalColor {
        TerminalColor::Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}
