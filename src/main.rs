use clap::{Arg, ArgAction, Command};
use color_thief::{Color, ColorFormat};
use regex::Regex;
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{self, stdout},
    path::Path,
    process,
    sync::{Arc, Mutex},
};

use url::Url;

use copypasta::ClipboardContext;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::time::Duration;

mod colors_canvas;
use colors_canvas::ColorsCanvas;

mod color_trait;
use color_trait::ColorTrait;

mod image_trait;
use image_trait::ImageTrait;

mod image_file_lib;
use image_file_lib::ImageFile;

mod cli_ui;
use cli_ui::{run_app, App, GaugeApp, GaugeAppGuard};

use io::ErrorKind;
use walkdir::WalkDir;

use rayon::prelude::*;

const IMAGES_EXTENSION_REGEX_PATTERN: &str =
    r"(?i)\.(png|jpe?g|gif|bmp|ico|tiff|webp|avif|pnm|dds|tga)";

fn main() -> io::Result<()> {
    /*
    CLI Program
     */
    let matches = Command::new("copycolors")
        .author("Abdoul ISSA BIDA <issamadjid1995@gmail.com>")
        .version("0.2.0")
        .about("Fast dominant colors extraction CLI")
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
        .arg(Arg::new("regex").long("regex").help("File regex pattern"))
        .arg(
            Arg::new("recursive")
                .long("recursive")
                .help("Browse folder recursively")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("rgb")
                .long("rgb")
                .short('r')
                .help("Print RGB code")
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
        Some(&Color { r: 0, g: 0, b: 0 })
    } else if bcw {
        Some(&Color {
            r: 255,
            g: 255,
            b: 255,
        })
    } else {
        None
    };

    let mut file_path = match matches.get_raw("file_path") {
        Some(f) => String::from(f.into_iter().next().unwrap().to_str().unwrap()),
        None => {
            eprintln!("You should put a directory path or a local or remote file path.");
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
            Ok(nb) => {
                if nb == 1 {
                    eprintln!("n should be > 1.");
                    process::exit(1);
                }
                nb
            }
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

    let repertory = fs::read_dir(&file_path);
    if repertory.is_ok() {
        /*
        Directory Case
        */
        let mut ctx = ClipboardContext::new().unwrap();
        let regex = matches.get_raw("regex");
        let regex = match regex {
            Some(r) => String::from(r.into_iter().next().unwrap().to_str().unwrap()),
            None => "".to_string(),
        };
        let recursive = matches.get_flag("recursive");
        let matching_files = get_matching_files(&file_path, &regex, recursive);
        if let Err(error_message) = matching_files {
            eprintln!("{}", error_message);
            process::exit(1);
        }

        let images_paths = matching_files.unwrap();
        let images_colors_map = Arc::new(Mutex::new(
            HashMap::<String, Result<Vec<Color>, String>>::with_capacity(images_paths.len()),
        ));
        let terminal = Arc::new(Mutex::new(ratatui::init()));
        let gauge_app = GaugeApp::default();
        let gauge = GaugeAppGuard::new(gauge_app);
        let inc: Arc<Mutex<u16>> = Arc::new(Mutex::new(0));
        let nb_images = images_paths.len() as u16;
        // Start the waiting gauge
        gauge.start();
        images_paths
            .into_par_iter()
            .map(|path| {
                (
                    path.to_owned(),
                    get_extracted_colors(path, nb_colors as u8, &excluded_colors, bc_color),
                )
            })
            .for_each(|(path, colors_result)| {
                let mut shared = images_colors_map.lock().unwrap();
                let path_copy = path.clone();
                shared.entry(path).or_insert(colors_result);
                terminal
                    .lock()
                    .unwrap()
                    .draw(|frame| frame.render_widget(&gauge, frame.area()))
                    .unwrap();

                gauge.handle_events().unwrap();
                let mut val = inc.lock().unwrap();
                let prop_val = (((*val) as f32 / nb_images as f32) * 100.0) as u16;
                gauge.update(prop_val, path_copy);
                *val += 1;
            });

        ratatui::restore();
        let images_colors_map: HashMap<String, Result<Vec<Color>, String>> =
            Arc::try_unwrap(images_colors_map)
                .expect("There are still other references to the Arc")
                .into_inner()
                .expect("Mutex cannot be locked");

        /*
        Sort alphabetically the files
        */
        let mut images_names = images_colors_map
            .iter()
            .map(|(k, _)| k)
            .collect::<Vec<&String>>();
        images_names.sort();
        let mut images_colors_map_sorted: BTreeMap<String, Result<Vec<Color>, String>> =
            BTreeMap::<String, Result<Vec<Color>, String>>::new();
        for image_name in images_names {
            images_colors_map_sorted
                .entry(image_name.clone())
                .or_insert(images_colors_map.get(image_name).unwrap().clone());
        }
        let images_colors_map = images_colors_map_sorted;
        if images_colors_map.is_empty() {
            let mut none_matching_files_message = String::from("No images file in that repository");
            if !regex.is_empty() {
                none_matching_files_message.push_str(" matching this regex pattern");
            }
            none_matching_files_message.push_str(".");
            eprintln!("{}", none_matching_files_message);
            process::exit(1);
        }

        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let tick_rate = Duration::from_millis(250);
        let app = App::new(
            images_colors_map,
            &file_path,
            nb_colors as u8,
            with_rgb,
            &excluded_colors,
            bc_color,
        );
        let res = run_app(&mut terminal, app, tick_rate, &mut ctx);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;

        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{err:?}");
        }
        ratatui::restore();
    } else {
        /*
        Image File Case
         */
        let image_path_extraction_regex = format!("(?P<link>.*{IMAGES_EXTENSION_REGEX_PATTERN})");
        let image_regex = Regex::new(&image_path_extraction_regex).unwrap();
        file_path = match image_regex.captures(&file_path) {
            Some(fp) => String::from(&fp["link"]),
            _ => {
                eprintln!(
                    "The path you enter is neither that of an existing repertory, nor that of an valid image file (with extension: .png, .jpeg, .jpg ...etc). Please check it, and try again."
                );
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
        let colors = image_file.get_colors_from_images(nb_colors as u8, &excluded_colors, bc_color);
        if let Err(extraction_error_message) = colors {
            eprintln!("{}", extraction_error_message);
            process::exit(1);
        }
        let colors = colors.unwrap();
        let cv = ColorsCanvas::new(colors, show_canvas, with_rgb, false);
        cv.display();
    }

    Ok(())
}

/// Repertory files matched a regex pattern
fn get_matching_files(
    directory: &str,
    regex_pattern: &str,
    recursive: bool,
) -> Result<Vec<String>, String> {
    let mut result_files = Vec::new();
    // Compile the regex pattern
    if Regex::new(regex_pattern).is_err() {
        return Err(format!("Invalid regex pattern: {regex_pattern}"));
    }
    let re = Regex::new(regex_pattern).unwrap();
    let images_re = Regex::new(IMAGES_EXTENSION_REGEX_PATTERN).unwrap();
    // Traverse the directory and filter files matching the regex pattern
    let read_directory = fs::read_dir(directory);
    if read_directory.is_err() {
        let error = read_directory.err().unwrap().kind();
        let dir_error_message = match error {
            ErrorKind::NotFound => format!(
                "Sorry, the directory: \"{directory}\" is not found. Please, make sure it exists."
            ),
            ErrorKind::PermissionDenied => {
                format!("You don't have the permissions to read the directory: \"{directory}\".")
            }
            _ => format!("Error, while reading the directory: \"{directory}\"."),
        };
        return Err(dir_error_message);
    }
    let mut walkdir = WalkDir::new(directory);
    if !recursive {
        walkdir = walkdir.max_depth(1);
    }
    let dir_entries = walkdir.into_iter().filter_map(Result::ok);
    for entry in dir_entries {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                if re.is_match(file_name) && images_re.is_match(file_name) {
                    result_files.push(path.display().to_string());
                }
            }
        }
    }
    Ok(result_files)
}

/// Extract colors from file path
/// used for iterated  extractions in repertory case
fn get_extracted_colors<'a>(
    file_path: String,
    nb_colors: u8,
    excluded_colors: &'a Vec<Color>,
    bc_color: Option<&'a Color>,
) -> Result<Vec<Color>, String> {
    let image_file = ImageFile::new(file_path);
    if image_file.image.is_err() {
        match image_file.image.err().unwrap() {
            image::ImageError::IoError(io_error) => match io_error.kind() {
                io::ErrorKind::NotFound => {
                    return Err(
                        "File not found.\nPlease be sure you provide the correct path!".to_string(),
                    );
                }
                _ => {
                    return Err("Error while opening the file!".to_string());
                }
            },
            _ => {
                return Err("Error while opening the file!".to_string());
            }
        }
    }
    // Colors extracting
    image_file.get_colors_from_images(nb_colors, excluded_colors, bc_color)
}

/// Download remote file
fn download_file(file_link: &str, file_dest: &str) {
    let mut resp = ureq::get(file_link)
        .call()
        .expect("request failed")
        .into_reader();
    let mut out = File::create(file_dest).expect("failed to create file");
    io::copy(&mut resp, &mut out).expect("failed to copy content");
}
