use color_thief::Color;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::style::Color as RatatuiColor, prelude::*, widgets::*};
use regex::Regex;
use std::{
    fs, io,
    time::{Duration, Instant},
};

use crate::ColorsCanvas;
use crate::ImageFile;

struct StatefulList<T> {
    state: ListState,
    vertical_scroll_state: ScrollbarState,
    vertical_scroll: usize,
    items: Vec<T>,
}

enum InputMode {
    Normal,
    Browsing,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            vertical_scroll_state: ScrollbarState::new(0),
            vertical_scroll: 0,
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
        self.vertical_scroll_state = self.vertical_scroll_state.position(i);
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
        self.vertical_scroll_state = self.vertical_scroll_state.position(i);
    }

    fn unselect(&mut self) {
        self.state.select(None);
        self.vertical_scroll_state = self.vertical_scroll_state.position(0);
    }
}

pub struct App {
    items: StatefulList<String>,
    dir_path: String,
    input_mode: InputMode,
    nb_colors: u8,
    with_rgb: bool,
    excluded_colors: Vec<Color>,
    bc_color: Option<Color>,
}

impl App {
    pub fn new(
        dir_path: &str,
        nb_colors: u8,
        with_rgb: bool,
        excluded_colors: Vec<Color>,
        bc_color: Option<Color>,
    ) -> App {
        let images_re =
            Regex::new(r"\.(png|jpe?g|gif|bmp|ico|tiff|webp|avif|pnm|dds|tga)$").unwrap();
        let files = fs::read_dir(dir_path).unwrap();
        let images_files = files
            .into_iter()
            .map(|f| f.unwrap().path().file_name().unwrap().to_owned())
            .map(|f| f.to_str().unwrap().to_string())
            .filter(|f| images_re.is_match(f))
            .collect::<Vec<String>>();
        let dir_path = dir_path.to_string();
        App {
            items: StatefulList::with_items(images_files),
            dir_path,
            input_mode: InputMode::Normal,
            nb_colors,
            with_rgb,
            excluded_colors,
            bc_color,
        }
    }
}

pub fn run_app<B: Backend>(
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
        let help_message =
            Paragraph::new(help_message).scroll((app.items.vertical_scroll as u16, 0));
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
            .highlight_style(
                Style::default()
                    .bg(RatatuiColor::White)
                    .fg(RatatuiColor::Black)
                    .add_modifier(Modifier::ITALIC),
            )
            .highlight_symbol("> ");
        app.items.vertical_scroll_state =
            app.items.vertical_scroll_state.content_length(items.len());

        // TODO: tester pb avec la modification et les refresh
        frame.render_stateful_widget(items, main_layout[1], &mut app.items.state);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            main_layout[1],
            &mut app.items.vertical_scroll_state,
        );
        let selected_item_index = app.items.state.selected();
        if selected_item_index.is_none() {
            let selected_item = "Please, select a image file to extract its colors.";
            frame.render_widget(Paragraph::new(selected_item.bold()), main_layout[2]);
        } else {
            let selected_item = &app.items.items[selected_item_index.unwrap()];
            let file_path = format!("{}/{}", app.dir_path, selected_item);
            let file_p = file_path.clone();
            let image_file = ImageFile::new(file_path);
            if image_file.image.is_err() {
                let error_message = match image_file.image.err().unwrap() {
                    image::ImageError::IoError(io_error) => match io_error.kind() {
                        io::ErrorKind::NotFound => {
                            "File not found.\nPlease be sure you provide the correct path!"
                        }
                        _ => "Error while opening the file!",
                    },
                    _ => "Error while opening the file!",
                };
                let error_message = format!("{}{}", file_p, error_message);
                frame.render_widget(
                    Paragraph::new(
                        error_message.set_style(
                            Style::new()
                                .bg(RatatuiColor::Rgb(255, 255, 0))
                                .fg(RatatuiColor::Rgb(0, 0, 0)),
                        ),
                    )
                    .block(Block::default().title("Error").borders(Borders::ALL)),
                    main_layout[2],
                );
            } else {
                // TODO: Adapt layout according to success or not
                let colors = image_file.get_colors_from_images(
                    app.nb_colors,
                    app.excluded_colors.to_owned(),
                    app.bc_color,
                );
                // TODO: Fix
                let cv = ColorsCanvas::new(colors, false, app.with_rgb, false);
                let tui_text = cv.tui_text();
                frame.render_widget(
                    List::new(tui_text).block(
                        Block::default()
                            .title("Extracted colors")
                            .borders(Borders::ALL),
                    ),
                    main_layout[2],
                );
            }
        }
    }
}
