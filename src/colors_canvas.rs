use crate::{io, Color, ColorTrait};

use crossterm::{
    style::{ Print, Stylize, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};

//  Canvas Structure
pub struct ColorsCanvas {
    colors: Vec<Color>,
    show_canvas: bool,
    with_rgb: bool,
}

impl ColorsCanvas {
    pub fn new(colors: Vec<Color>, sc: bool, wrgb: bool) -> Self {
        ColorsCanvas {
            colors,
            show_canvas: sc,
            with_rgb: wrgb,
        }
    }

    fn print(&self) {
        let t_colors = vec![
            Color { r: 0, g: 0, b: 0 },
            Color {
                r: 255,
                g: 255,
                b: 255,
            },
        ];
        for i in 0..self.colors.len() {
            let col = self.colors[i];
            let txt_col = col.best_contrast(&t_colors);
            let color_str = match self.with_rgb {
                true => col.rgb_str(),
                _ => col.hexadecimal_str(),
            };
            stylize_text(color_str, true, txt_col,&col);
            if i < self.colors.len() - 1 {
                print!(",");
            }
        }
        println!();
    }

    fn draw(&self) {
        let (term_w, _) = crossterm::terminal::size().unwrap();
        // Square positioning
        let nb_square: u32 = self.colors.len() as u32;
        let width: u32 = term_w as u32;
        let square_c: u32 = match self.with_rgb {
            true => 8,
            _ => 4,
        };

        let r_spacing = 2; // row spacing
        let c_spacing = 2; // column spacing

        let nb_col = width / (2 * square_c + c_spacing);
        let nb_row = nb_square as f32 / nb_col as f32;
        let nb_row = nb_row.ceil() as u32;

        for i in 0..nb_row * (square_c + r_spacing) {
            let mut line_nb_squares = nb_col;
            if i / (square_c + r_spacing) == nb_row - 1 && nb_square != nb_row * nb_col {
                line_nb_squares = nb_square % nb_col;
                let r_squares = nb_col - line_nb_squares;
                let m = r_squares * (2 * square_c + c_spacing) / 2;
                print!("{}", " ".repeat(m as usize));
            }
            for j in 0..line_nb_squares * (2 * square_c + c_spacing) {
                let r = i / (square_c + r_spacing);
                let c = j / (2 * square_c + c_spacing);
                let index = r * nb_col + c;

                let r_mod = i % (square_c + r_spacing);
                let c_mod = j % (2 * square_c + c_spacing);
                if (r_mod < r_spacing) || (c_mod < c_spacing) {
                    // if r_mod == r_spacing - 1 && index < nb_square {
                    //     if c_mod >= c_spacing {
                    //         print!("_");
                    //     } else {
                    //         print!(" ");
                    //     }
                    // } else
                    if r_mod == r_spacing - 1 {
                        if c_mod == 0 && index < nb_square {
                            let color_str = match self.with_rgb {
                                true => self.colors[index as usize].rgb_str(),
                                _ => self.colors[index as usize].hexadecimal_str(),
                            };

                            if c < nb_col && index < nb_square {
                                let color_str = format!("{}{}{}",
                                " ".repeat(c_spacing as usize),color_str.as_str().bold(), 
                                " ".repeat(2 * square_c as usize - color_str.len()));

                                io::stdout()
                                .execute(Print(color_str)).unwrap();
                           
                            }
                        } else {
                            print!("");
                        }
                    } else {
                        print!(" ");
                    }
                } else if index < nb_square {
                        let colour = self.colors[index as usize];
                        // print!("{}", " ".on_truecolor(colour.r, colour.g, colour.b));
                        stylize_text(" ".to_string(), false, &colour, &colour);
                }
            }
            println!();
        }
        println!();
    }

    pub fn display(&self) {
        match self.show_canvas {
            true => self.draw(),
            _ => self.print(),
        }
    }
}

fn stylize_text(text: String, bold: bool,fg: &Color, bg: &Color) { 
    if bold {
    io::stdout()
    .execute(SetForegroundColor(fg.to_term_color())).unwrap()
    .execute(SetBackgroundColor(bg.to_term_color())).unwrap()
    .execute(Print(text.bold())).unwrap()
    .execute(ResetColor).unwrap();   
    } else { 
        io::stdout()
        .execute(SetForegroundColor(fg.to_term_color())).unwrap()
        .execute(SetBackgroundColor(bg.to_term_color())).unwrap()
        .execute(Print(text)).unwrap()
        .execute(ResetColor).unwrap();   
    }
}