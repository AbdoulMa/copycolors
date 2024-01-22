use crate::{io, Color, ColorTrait, Line, Span, Style, Text};

use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::{
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor, Stylize},
    ExecutableCommand,
};
use ratatui::style::Modifier;

pub struct ColorsCanvas {
    colors: Vec<Color>,
    show_canvas: bool,
    with_rgb: bool,
    clip_colors: bool,
}

impl ColorsCanvas {
    pub fn new(colors: Vec<Color>, sc: bool, wrgb: bool, clip_colors: bool) -> Self {
        ColorsCanvas {
            colors,
            show_canvas: sc,
            with_rgb: wrgb,
            clip_colors,
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
        //  Clipboard management
        let mut ctx = ClipboardContext::new().unwrap();
        let mut colors_clipped_text = String::new();
        for i in 0..self.colors.len() {
            let col = self.colors[i];
            let txt_col = col.best_contrast(&t_colors);
            let color_str = match self.with_rgb {
                true => col.rgb_str(),
                _ => col.hexadecimal_str(),
            };
            colors_clipped_text.push_str(&color_str);
            stylize_text(color_str, true, txt_col, &col);
            if i < self.colors.len() - 1 {
                print!(",");
                colors_clipped_text.push(',');
            }
        }
        println!();
        // Clip colors if flagged
        if self.clip_colors {
            ctx.set_contents(colors_clipped_text).unwrap();
            let _clipped_colors = ctx.get_contents().unwrap();
        }
    }

    pub fn print_tui(&self) -> Text {
        let mut lines_result = Line::from(vec![]);
        let t_colors = vec![
            Color { r: 0, g: 0, b: 0 },
            Color {
                r: 255,
                g: 255,
                b: 255,
            },
        ];
        //  Clipboard management
        let mut ctx = ClipboardContext::new().unwrap();
        let mut colors_clipped_text = String::new();
        for i in 0..self.colors.len() {
            let col = self.colors[i];
            let txt_col = col.best_contrast(&t_colors);
            let color_str = match self.with_rgb {
                true => col.rgb_str(),
                _ => col.hexadecimal_str(),
            };
            colors_clipped_text.push_str(&color_str);
            //   TODO: change here
            //  stylize_text(color_str, true, txt_col, &col);
            let span = Span::styled(
                color_str,
                Style::new()
                    .fg(ratatui::style::Color::Rgb(txt_col.r, txt_col.g, txt_col.b))
                    .bg(ratatui::style::Color::Rgb(col.r, col.g, col.b))
                    .add_modifier(Modifier::BOLD),
            );
            lines_result.spans.push(span);
            if i < self.colors.len() - 1 {
                lines_result.spans.push(Span::raw(","));
                // print!(",");
                colors_clipped_text.push(',');
            }
        }
        //    println!();
        // Clip colors if flagged
        if self.clip_colors {
            ctx.set_contents(colors_clipped_text).unwrap();
            let _clipped_colors = ctx.get_contents().unwrap();
        }
        Text::from(vec![lines_result])
    }

    fn draw(&self) {
        //  Clipboard management
        let mut ctx = ClipboardContext::new().unwrap();
        let mut colors_clipped_text = String::new();

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
                    if r_mod == r_spacing - 1 {
                        if c_mod == 0 && index < nb_square {
                            let color_str = match self.with_rgb {
                                true => self.colors[index as usize].rgb_str(),
                                _ => self.colors[index as usize].hexadecimal_str(),
                            };
                            colors_clipped_text.push_str(&color_str);
                            if index < nb_square - 1 {
                                colors_clipped_text.push(',');
                            }
                            if c < nb_col && index < nb_square {
                                let color_str = format!(
                                    "{}{}{}",
                                    " ".repeat(c_spacing as usize),
                                    color_str.as_str().bold(),
                                    " ".repeat(2 * square_c as usize - color_str.len())
                                );

                                io::stdout().execute(Print(color_str)).unwrap();
                            }
                        } else {
                            print!("");
                        }
                    } else {
                        print!(" ");
                    }
                } else if index < nb_square {
                    let colour = self.colors[index as usize];
                    stylize_text(" ".to_string(), false, &colour, &colour);
                }
            }
            println!();
        }
        println!();
        // Clip colors if flagged
        if self.clip_colors {
            ctx.set_contents(colors_clipped_text).unwrap();
            let _clipped_colors = ctx.get_contents().unwrap();
        }
    }

    pub fn display(&self) {
        match self.show_canvas {
            true => self.draw(),
            _ => self.print(),
        }
    }

    // TODO: Print and drawing methods for colors exctracted from
    // Text and styling
    // fn interactive_print() -> Text
    // fn interactive_draw() -> Text
}

fn stylize_text(text: String, bold: bool, fg: &Color, bg: &Color) {
    if bold {
        io::stdout()
            .execute(SetForegroundColor(fg.to_term_color()))
            .unwrap()
            .execute(SetBackgroundColor(bg.to_term_color()))
            .unwrap()
            .execute(Print(text.bold()))
            .unwrap()
            .execute(ResetColor)
            .unwrap();
    } else {
        io::stdout()
            .execute(SetForegroundColor(fg.to_term_color()))
            .unwrap()
            .execute(SetBackgroundColor(bg.to_term_color()))
            .unwrap()
            .execute(Print(text))
            .unwrap()
            .execute(ResetColor)
            .unwrap();
    }
}
