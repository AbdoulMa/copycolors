use color_thief::Color;
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{prelude::style::Color as RatatuiColor, prelude::*, widgets::*, DefaultTerminal};
use std::{
    collections::BTreeMap,
    error::Error,
    io, process,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use Constraint::{Length, Ratio};

use crate::ColorsCanvas;
use crate::ImageFile;

const TERMINAL_GREEN: RatatuiColor = RatatuiColor::Rgb(124, 252, 0);

/// Gauge
#[derive(Debug, Default, Clone)]
pub struct GaugeApp {
    pub state: GaugeAppState,
    progress: f64,
    current_file: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GaugeAppState {
    #[default]
    Running,
    Started,
    Quitting,
}

impl GaugeApp {
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn Error>> {
        while self.state != GaugeAppState::Quitting {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn update(&mut self, inc: u16, path: String) {
        if self.state != GaugeAppState::Started {
            return;
        }

        self.current_file = path;
        self.progress = inc as f64;
        if self.progress >= 100.0 {
            self.progress = 100.0;
            // Optionally set to quitting state here, or let the user quit manually
            self.state = GaugeAppState::Quitting;
        }
    }

    pub fn handle_events(&mut self) -> Result<(), Box<dyn Error>> {
        self.start();
        let timeout = Duration::from_secs_f32(1.0 / 15.0); // latency
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => self.quit(),
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                self.quit();
                            }
                        }
                        _ => self.start(),
                    }
                }
            }
        }
        Ok(())
    }

    fn start(&mut self) {
        self.state = GaugeAppState::Started;
    }

    fn quit(&mut self) {
        self.state = GaugeAppState::Quitting;
        ratatui::restore();
        process::exit(1);
    }
}

impl Widget for &GaugeApp {
    #[allow(clippy::similar_names)]
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}

/// GaugeAppGuard for parallel shared data protection
pub struct GaugeAppGuard(pub Arc<Mutex<GaugeApp>>);

impl GaugeAppGuard {
    pub fn new(gauge_app: GaugeApp) -> Self {
        GaugeAppGuard(Arc::new(Mutex::new(gauge_app)))
    }

    pub fn handle_events(&self) -> Result<(), Box<dyn Error>> {
        self.0.lock().unwrap().handle_events()
    }

    pub fn update(&self, inc: u16, path: String) {
        self.0.lock().unwrap().update(inc, path);
    }

    pub fn is_quitting(&self) -> bool {
        self.0.lock().unwrap().state == GaugeAppState::Quitting
    }

    pub fn start(&self) {
        self.0.lock().unwrap().start();
    }
}

impl Widget for &GaugeAppGuard {
    #[allow(clippy::similar_names)]
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::vertical([Length(17)]);
        let [gauge_area] = layout.areas(area);

        let layout = Layout::vertical([Ratio(1, 3)]);
        let [gauge_area] = layout.areas(gauge_area);

        self.0.lock().unwrap().render_gauge(gauge_area, buf);
    }
}
fn title_block<'a>(title: &'a str, current_file: &'a str) -> Block<'a> {
    let title = Line::from(Span::from(title))
        .style(Style::new().bold())
        .alignment(Alignment::Left);
    let file_title = Line::from(Span::from(current_file)).alignment(Alignment::Right);
    Block::new()
        .borders(Borders::NONE)
        .padding(Padding::vertical(1))
        .title(file_title)
        .title(title)
        .fg(RatatuiColor::Rgb(255, 255, 255))
}

impl GaugeApp {
    fn render_gauge(&self, area: Rect, buf: &mut Buffer) {
        let title = title_block("Loading, please wait ... ", &self.current_file);
        let label = Span::styled(
            format!("{:.1} %", self.progress),
            Style::new()
                .italic()
                .bold()
                .fg(RatatuiColor::Rgb(255, 255, 255)),
        );
        Gauge::default()
            .block(title)
            .gauge_style(TERMINAL_GREEN)
            .label(label)
            .ratio(self.progress / 100.0)
            .render(area, buf);
    }
}

/// ListApp
struct StatefulList<T> {
    state: ListState,
    vertical_scroll_state: ScrollbarState,
    vertical_scroll: usize,
    items: Vec<T>,
    start_time: Instant,
    clip_color: bool,
    select_more: bool,
    select_less: bool,
}

#[derive(PartialEq, Eq)]
enum InputMode {
    Normal,
    Browsing,
}

impl<T> StatefulList<T> {
    fn get_start_time(&self) -> Instant {
        self.start_time
    }

    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            vertical_scroll_state: ScrollbarState::new(0),
            vertical_scroll: 0,
            items,
            start_time: Instant::now(),
            clip_color: false,
            select_more: false,
            select_less: false,
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
        self.clip_color = false;
        self.select_less = false;
        self.select_more = false;
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
        self.clip_color = false;
        self.select_less = false;
        self.select_more = false;
    }

    fn unselect(&mut self) {
        self.state.select(None);
        self.vertical_scroll_state = self.vertical_scroll_state.position(0);
    }
}

/// Images browsing app
pub struct App<'a> {
    images_paths: BTreeMap<String, Result<Vec<Color>, String>>,
    items: StatefulList<String>,
    input_mode: InputMode,
    nb_colors: u8,
    nb_extracted_colors: u8,
    with_rgb: bool,
    excluded_colors: &'a Vec<Color>,
    bc_color: Option<&'a Color>,
}

impl<'a> App<'a> {
    pub fn new(
        images_paths: BTreeMap<String, Result<Vec<Color>, String>>,
        dir_path: &str,
        nb_colors: u8,
        with_rgb: bool,
        excluded_colors: &'a Vec<Color>,
        bc_color: Option<&'a Color>,
    ) -> App<'a> {
        let dir_path = if !dir_path.ends_with("/") {
            format!("{dir_path}/")
        } else {
            dir_path.to_string()
        };
        let images_items = images_paths
            .iter()
            .map(|(file, _)| file.replace(&dir_path, ""))
            .collect();
        App {
            images_paths,
            items: StatefulList::with_items(images_items),
            input_mode: InputMode::Normal,
            nb_colors,
            nb_extracted_colors: nb_colors,
            with_rgb,
            excluded_colors,
            bc_color,
        }
    }

    fn increment_nb_extracted_colors(&mut self) {
        // interactively increment number of colors extracted up to 10
        if self.nb_extracted_colors < 10 {
            self.nb_extracted_colors += 1;
        }
    }

    fn decrement_nb_extracted_colors(&mut self) {
        // interactively decrement number of colors extracted up to 2
        if self.nb_extracted_colors > 2 {
            self.nb_extracted_colors -= 1;
        }
    }
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
    ctx: &mut ClipboardContext,
) -> io::Result<()> {
    let last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app, ctx))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
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
                                KeyCode::Char('c') | KeyCode::Char('C') => {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        app.items.start_time = Instant::now();
                                        app.items.clip_color = true;
                                    }
                                }
                                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                                KeyCode::Char('l') | KeyCode::Char('L') => {
                                    app.items.select_less = true;
                                    app.decrement_nb_extracted_colors()
                                }
                                KeyCode::Char('m') | KeyCode::Char('M') => {
                                    app.items.select_more = true;
                                    app.increment_nb_extracted_colors()
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

fn green_terminal_text(text: &str) -> Span<'_> {
    Span::styled(
        text,
        Style::new().fg(TERMINAL_GREEN).add_modifier(Modifier::BOLD),
    )
}
fn ui(frame: &mut Frame, app: &mut App, ctx: &mut ClipboardContext) {
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(9),
            Constraint::Min(0),
        ],
    )
    .split(frame.area());

    let help_message = match app.input_mode {
        InputMode::Normal => vec![
            "Press ".into(),
            "[".into(),
            green_terminal_text("↓/↑"),
            "] ".into(),
            "to browse the files. ".into(),
            "Or press ".into(),
            "[".into(),
            green_terminal_text("q"),
            "] ".into(),
            "to exit.".into(),
        ],
        InputMode::Browsing => vec![
            "Press ".into(),
            "[".into(),
            green_terminal_text("m"),
            "] ".into(),
            "to extract more, ".into(),
            "[".into(),
            green_terminal_text("l"),
            "] ".into(),
            "to less, ".into(),
            "[".into(),
            green_terminal_text("Ctrl+c"),
            "] ".into(),
            "to copy or ".into(),
            "[".into(),
            green_terminal_text("q"),
            "] ".into(),
            "to exit.".into(),
        ],
    };

    let help_message = Text::from(Line::from(help_message));
    let help_message = Paragraph::new(help_message).scroll((app.items.vertical_scroll as u16, 0));
    frame.render_widget(help_message, main_layout[0]);
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|path| ListItem::new(path.clone()).style(Style::default().fg(RatatuiColor::White)))
        .collect();
    let mut items_list_title = Vec::<Span>::with_capacity(2);
    items_list_title.push(Span::raw("Files"));
    if app.input_mode == InputMode::Browsing {
        items_list_title.push(Span::raw(format!(
            " · # {}/{} ",
            app.items.state.selected().unwrap() + 1,
            items.len()
        )));
    }
    let items = List::new(items)
        .block(
            Block::default()
                .title(Line::from(items_list_title))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(RatatuiColor::White))
        .highlight_style(
            Style::default()
                .bg(RatatuiColor::White)
                .fg(RatatuiColor::Black)
                .add_modifier(Modifier::ITALIC),
        )
        .highlight_symbol("> ");
    app.items.vertical_scroll_state = app.items.vertical_scroll_state.content_length(items.len());

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
        let selected_item_index = selected_item_index.unwrap();
        let selected_item = (&app.items.items[selected_item_index]).to_string();

        let colors_result = app.images_paths.iter().nth(selected_item_index).unwrap();
        let colors_result = colors_result.1.to_owned();
        if let Err(error_message) = colors_result {
            frame.render_widget(
                Paragraph::new(error_message)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
                    .block(Block::default().title("Error").borders(Borders::ALL)),
                main_layout[2],
            );
        } else {
            let colors = colors_result.unwrap();
            let cv = ColorsCanvas::new(colors, false, app.with_rgb, app.items.clip_color);
            let duration = Duration::from_secs(3); // clipboarding message duration

            let mut item = if !app.items.select_less
                && !app.items.select_more
                && app.items.clip_color
                && app.items.get_start_time().elapsed() <= duration
            {
                ctx.set_contents(cv.colors_clipboarded().unwrap()).unwrap();
                format!("{} - copied to clipboard!", selected_item)
            } else {
                selected_item.to_string()
            };
            let cv_b;
            let tui_text = if !app.items.select_less && !app.items.select_more {
                app.nb_extracted_colors = app.nb_colors;
                cv.tui_text()
            } else {
                // Manage more/less colors extraction
                let file_path = app
                    .images_paths
                    .iter()
                    .nth(selected_item_index)
                    .unwrap()
                    .0
                    .to_owned();
                let file_p = file_path.clone();
                let image_file = ImageFile::new(file_path);
                if image_file.image.is_err() {
                    let error_message = match image_file.image.err().unwrap() {
                        image::ImageError::IoError(io_error) => match io_error.kind() {
                            io::ErrorKind::NotFound => "File not found.",
                            _ => "Error while opening the file!",
                        },
                        _ => "Error while opening the file!",
                    };
                    let error_message = format!("{}: {}", file_p, error_message);
                    vec![Span::raw(error_message)]
                } else {
                    let colors = image_file.get_colors_from_images(
                        app.nb_extracted_colors,
                        app.excluded_colors,
                        app.bc_color,
                    );
                    if let Err(error_message) = colors.as_ref() {
                        vec![Span::raw(error_message)];
                    }

                    let colors = colors.unwrap();
                    cv_b = ColorsCanvas::new(colors, false, app.with_rgb, app.items.clip_color);
                    // Manage clip boarding
                    if app.items.clip_color && app.items.get_start_time().elapsed() <= duration {
                        ctx.set_contents(cv_b.colors_clipboarded().unwrap())
                            .unwrap();
                        item = format!("{} - copied to clipboard !", selected_item);
                    };
                    cv_b.tui_text()
                }
            };
            let colors_extraction_canva = vec![
                Line::from(vec![]),
                Line::from(vec![Span::raw(item)]),
                Line::from(vec![]),
                Line::from(tui_text),
            ];
            frame.render_widget(
                Paragraph::new(colors_extraction_canva)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true }),
                main_layout[2],
            );
        }
    }
}
