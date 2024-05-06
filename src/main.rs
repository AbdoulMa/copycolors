// TODO https://kerkour.com/rust-cross-compilation
//  Terminal color, alternative termcolor crates
use clap::{Arg, ArgAction, Command};
use color_thief::{Color, ColorFormat};
use image::{ImageError, Rgb};
use regex::Regex;
use std::{
    cmp::Ordering,
    fs::{self, File}, // File & Repertory management
    path::Path,
    process,
};

use url::Url;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::style::Color as RatatuiColor, prelude::*, widgets::*};
use std::io::{self, stdout};
use std::time::{Duration, Instant};
mod colors_canvas;
use colors_canvas::ColorsCanvas;

mod color_trait;
use color_trait::ColorTrait;

mod image_trait;
use image_trait::ImageTrait;

mod image_file_lib;
use image_file_lib::ImageFile;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

//  TODO: Wrap it in a module
enum InputMode {
    Normal,
    Browsing,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App {
    // items: StatefulList<&'a str>,
    items: StatefulList<String>,
    output: String,
    input_mode: InputMode,
    // events: Vec<(&'a str, &'a str)>,
}

impl App {
    fn new(dir_path: &str) -> App {
        let images_re =
            Regex::new(r"\.(png|jpe?g|gif|bmp|ico|tiff|webp|avif|pnm|dds|tga)$").unwrap();
        let files = fs::read_dir(dir_path).unwrap();
        let images_files = files
            .into_iter()
            .map(|f| f.unwrap().path().file_name().unwrap().to_owned())
            .map(|f| f.to_str().unwrap().to_string())
            .filter(|f| images_re.is_match(f))
            .collect::<Vec<String>>();

        App {
            items: StatefulList::with_items(images_files),
            output: String::new(),
            input_mode: InputMode::Normal,
        }
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Down => {
                                    app.input_mode = InputMode::Browsing;
                                    app.items.next();
                                }
                                KeyCode::Up => {
                                    app.input_mode = InputMode::Browsing;
                                    app.items.previous();
                                }
                                _ => {}
                            }
                        }
                    }
                    InputMode::Browsing => {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Left => {
                                    app.input_mode = InputMode::Normal;
                                    app.items.unselect();
                                }
                                KeyCode::Down => {
                                    app.items.next();
                                }
                                KeyCode::Up => {
                                    app.items.previous();
                                }
                                KeyCode::Char('q') => return Ok(()),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        // if last_tick.elapsed() >= tick_rate {
        //     app.on_tick();
        //     last_tick = Instant::now();
        // }
    }
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

// https://ratatui.rs/how-to/layout/
fn ui(frame: &mut Frame, app: &mut App) {
    if app.items.items.is_empty() {
        let no_files_message = vec![
            "No images files in that repository. ".into(),
            "Press ".into(),
            "q ".bold(),
            "to exit.".into(),
        ];

        let no_files_message = Text::from(Line::from(no_files_message));
        let no_files_message = Paragraph::new(no_files_message);
        frame.render_widget(no_files_message, frame.size());
    } else {
        let main_layout = Layout::new()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(9),
                Constraint::Min(0),
            ])
            .split(frame.size());

        let help_message = match app.input_mode {
            InputMode::Normal => vec![
                "Press ".into(),
                "↓/↑ ".bold(),
                "to browse the files. ".into(),
                "Or press ".into(),
                "q ".bold(),
                "to exit.".into(),
            ],
            InputMode::Browsing => vec!["Press ".into(), "q ".bold(), "to exit.".into()],
        };

        let help_message = Text::from(Line::from(help_message));
        let help_message = Paragraph::new(help_message);
        frame.render_widget(help_message, main_layout[0]);
        let items: Vec<ListItem> = app
            .items
            .items
            .iter()
            .map(|path| ListItem::new(path.clone()).style(Style::default().fg(RatatuiColor::White)))
            .collect();
        let items = List::new(items)
            .block(Block::default().title("Files").borders(Borders::ALL))
            .style(Style::default().fg(RatatuiColor::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        // TODO: tester pb avec la modification et les refresh
        frame.render_stateful_widget(items, main_layout[1], &mut app.items.state);
        let selected_item_index = app.items.state.selected();

        if selected_item_index.is_none() {
            let selected_item = "Please, select a image file to extract its colors.";
            frame.render_widget(Paragraph::new(selected_item.bold()), main_layout[2]);
        } else {
            let selected_item = &app.items.items[selected_item_index.unwrap()];
            frame.render_widget(
                Paragraph::new(
                    selected_item.as_str().set_style(
                        Style::new()
                            .bg(RatatuiColor::Rgb(255, 255, 0))
                            .fg(RatatuiColor::Rgb(0, 0, 0)),
                    ),
                )
                .block(
                    Block::default()
                        .title("Extracted Colors")
                        .borders(Borders::ALL),
                ),
                main_layout[2],
            );
        }
    }
}

/* Interactive CLI */
// fn handle_events() -> io::Result<bool> {
//     if event::poll(std::time::Duration::from_millis(50))? {
//         if let Event::Key(key) = event::read()? {
//             if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
//                 return Ok(true);
//             }
//         }
//     }
//     Ok(false)
// }

// fn ui(frame: &mut Frame) {
//     let test_canvas = ColorsCanvas::new(
//         vec![Color {
//             r: 241,
//             b: 25,
//             g: 87,
//         }],
//         false,
//         false,
//         false,
//     );
//     frame.render_widget(
//         Paragraph::new(test_canvas.print_tui())
//             .block(Block::default().title("Greeting").borders(Borders::ALL)),
//         frame.size(),
//     );
// }

fn main() -> io::Result<()> {
    /*
    CLI Program
     */
    let matches = Command::new("copycolors")
        .author("Abdoul ISSA BIDA <issamadjid1995@gmail.com>")
        .version("0.1.0")
        .about("Fast dominant colors extraction CLI")
        // TODO: Adapt when it is a directory
        .arg(
            Arg::new("file_path")
                .value_name("DIR_OR_FILE_PATH")
                .required(true)
                .help("Local repertory / Local or remote image path"),
        )
        .arg(
            Arg::new("nb-colors")
                .long("nb-colors")
                .short('n')
                .num_args(1)
                .default_value("5")
                .help("Specify the number of colors to extract"),
        )
        .arg(
            Arg::new("rgb")
                .long("rgb")
                .short('r')
                .help("Print RGB code")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("clip")
                .long("clip")
                .help("Clipboard colors extracted")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("exc-colors")
                .value_name("COLOURS")
                .long("exc-colors")
                .short('e')
                .next_line_help(true)
                .num_args(1..=5)
                .help(
                    r"Colors to exclude in hexadecimal
You can exclude up to 5 colors.                    
Ex: -e '#000000' '#FFFFFF'
                ",
                ),
        )
        .arg(
            Arg::new("canvas")
                .short('c')
                .long("canvas")
                .help("Show colors canvas")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("bcw")
                .long("bcw")
                .help(
                    "Order extracted colors from the best contrasting
with white to the less.",
                )
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("bcb")
                .long("bcb")
                .help(
                    r"Order extracted colors from the best contrasting
with black to the less.
When bcw & bcb are  both requested, bcb is used.",
                )
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let show_canvas = matches.get_flag("canvas");
    let with_rgb = matches.get_flag("rgb");
    let excluded_colors = if let Some(ec) = matches.get_many::<String>("exc-colors") {
        ec.map(|hex| hex.as_str())
            .map(|hex| Color::hex_to_rgb(hex).unwrap())
            .collect::<Vec<Color>>()
    } else {
        vec![]
    };

    let bcw = matches.get_flag("bcw");
    let bcb = matches.get_flag("bcb");
    let bc_color = if bcb {
        Some(Color { r: 0, g: 0, b: 0 })
    } else if bcw {
        Some(Color {
            r: 255,
            g: 255,
            b: 255,
        })
    } else {
        None
    };

    let clip_colors = matches.get_flag("clip");
    let mut file_path = match matches.get_raw("file_path") {
        Some(f) => String::from(f.into_iter().next().unwrap().to_str().unwrap()),
        None => {
            eprintln!("You should put a directory path or a local or remote file path.");
            process::exit(1);
        }
    };
    let repertory = fs::read_dir(&file_path);
    if repertory.is_ok() {
        // TODO: Manage Directory Case

        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let tick_rate = Duration::from_millis(250);
        let app = App::new(&file_path);
        let res = run_app(&mut terminal, App::new(&file_path) /* app*/, tick_rate);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;

        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{err:?}");
        }
    } else {
        // Image File Case
        let image_regex =
            Regex::new(r"(?P<link>.*(?i)\.(png|jpe?g|gif|bmp|ico|tiff|webp|avif|pnm|dds|tga))")
                .unwrap();
        file_path = match image_regex.captures(&file_path) {
            Some(fp) => String::from(&fp["link"]),
            _ => {
                // TODO: Change message
                eprintln!(
                "The path you enter is neither that of a valid repertory, nor that of a valid image file (with extension: .png, .jpeg, .jpg ...etc). Please check it, and try again."
            );
                process::exit(1);
            }
        };

        let nb_colors = match matches.get_raw("nb-colors") {
            Some(nb) => match nb
                .into_iter()
                .next()
                .unwrap()
                .to_str()
                .unwrap()
                .parse::<u32>()
            {
                Ok(nb) => nb,
                Err(_) => {
                    eprintln!("You should provide a valid positive number as second arguement.");
                    process::exit(1);
                }
            },
            None => {
                eprintln!("You should provide a valid number of colors you want to extract.");
                process::exit(1);
            }
        };

        // Check if it an url
        let url_parse = Url::parse(&file_path);
        // File is dropped with dir after the variable goes out of scope
        let dir = tempfile::tempdir().unwrap();
        if url_parse.is_ok() {
            let remote_file_name = Path::new(&file_path).file_name().unwrap().to_str().unwrap();
            // Create a directory inside of `std::env::temp_dir()`.
            let tmp_path = dir.path().join(remote_file_name);
            let tmp_path = tmp_path.as_path().display().to_string();
            download_file(&file_path, &tmp_path);
            file_path = tmp_path;
        }

        let image_file = ImageFile::new(file_path);
        if image_file.image.is_err() {
            match image_file.image.err().unwrap() {
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
            }
        }
        // Colors extracting
        let colors = image_file.get_colors_from_images(nb_colors as u8, excluded_colors, bc_color);
        let cv = ColorsCanvas::new(colors, show_canvas, with_rgb, clip_colors);
        cv.display();
    }

    Ok(())
}

fn download_file(file_link: &str, file_dest: &str) {
    let mut resp = ureq::get(file_link)
        .call()
        .expect("request failed")
        .into_reader();
    let mut out = File::create(file_dest).expect("failed to create file");
    io::copy(&mut resp, &mut out).expect("failed to copy content");
}
