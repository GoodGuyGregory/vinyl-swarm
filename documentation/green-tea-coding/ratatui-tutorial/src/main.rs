use std::{
    io, 
    thread,
    sync::mpsc,
    time::Duration,
};
use crossterm::{
    event::{KeyEvent, KeyCode, KeyEventKind},
};
use ratatui::{
    widgets::{Widget, Gauge, Block},
    style::{Color, Style},
    symbols::border,
    text::{Line},
    DefaultTerminal, Frame,
    prelude::{Layout, Rect,  Constraint, Stylize}
};

pub struct App {
    exit: bool,
    progress_bar_color: Color,
    background_progress: f64,    
}

// event Enums
enum Event {
    Input(crossterm::event::KeyEvent),
    Progress(f64),
}

fn handle_input_events(tx: mpsc::Sender<Event> ) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            _ => {}
        }
    }
}

fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut progress = 0_f64;
    let increment = 0.01_f64;
    loop {
        thread::sleep(Duration::from_millis(100));
        progress += increment;
        progress = progress.min(1_f64);
        tx.send(Event::Progress(progress)).unwrap();
    }
}

impl App {
    // 
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Progress(progress) => self.background_progress = progress,
            }
            // draw the changed UI 
            // takes a frame and draws something
            terminal.draw( |frame| self.draw(frame) )?;
        }

        ();
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    // handles key events: handles the App properties
    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('q') {
            self.exit = true;
        } else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('c') {
            // change the color of the progress bar after checking the current color
            // colors in Ratatui: https://ratatui.rs/examples/style/colors/
            if self.progress_bar_color == Color::Green {
                self.progress_bar_color = Color::Magenta;
            } else {
                self.progress_bar_color = Color::Green;
            }
        } else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('r') {
            self.background_progress = 1_f64;
        }
        Ok(())
    }

    

    
}

// implement the Widget Trait
impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where 
        Self: Sized 
        {
            let vertical_layout = Layout::vertical([Constraint::Percentage(20),Constraint::Percentage(80)]);

            // have to determine the area to render the widget
            let [title_area, gauge_area] = vertical_layout.areas(area);
            // render the title on the top
            Line::from("Process overview").bold().render(title_area, buf);

            let instructions = Line::from(vec![
                " Change color ".into(),
                "<C>".blue().bold(),
                " Restart Progress ".into(),
                "<R>".yellow().bold(),
                " Quit ".into(),
                "<Q>".blue().bold(),
            ])
            .centered();

            let block = Block::bordered()
                            .title(Line::from(" Background Processes "))
                            .title_bottom(instructions)
                            .border_set(border::THICK);


            // create the progress bar
            let progress_bar = Gauge::default()
                                .gauge_style(Style::default().fg(self.progress_bar_color))
                                .block(block)
                                .label(format!("Process 1: {:.2}%", self.background_progress * 100_f64))
                                .ratio(self.background_progress);

            progress_bar.render(Rect {
                x: gauge_area.left(),
                y: gauge_area.top(),
                width: gauge_area.width,
                height: 3
            }, buf);
        }
}

fn main() -> io::Result<()> {
    // initialize ratatui:
    // creates an instance of ratatui and puts it into raw mode
    let mut terminal = ratatui::init();

    let mut app =  App { exit: false, progress_bar_color: Color::Green, background_progress: 0_f64,};

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_to_input_event = event_tx.clone();
    // handle input events
    thread::spawn(move || {
        handle_input_events(tx_to_input_event);
    });

    // handle the progress events
    let tx_to_background_progress_events = event_tx.clone();
    thread::spawn(move || {
        run_background_thread(tx_to_background_progress_events)
    });

    let app_result = app.run(&mut terminal, event_rx);
    // returns the terminal to normal mode post exit
    ratatui::restore();
    app_result
}