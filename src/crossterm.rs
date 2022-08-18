use crate::print::Print;
use crate::{app::App, ui};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

pub fn run(tick_rate: Duration, paths: Vec<String>) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new("rsreit", paths.to_vec());
    let print = Print::default();
    let res = run_app(&mut terminal, app, print, tick_rate);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<'a, B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App<'a>,
    mut print: Print<'a>,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    for path in &app.paths {
        app.files.add(path.to_string(), &mut app.tabs);
    }

    app.sync_file(&mut print);
    loop {
        terminal.draw(|f| ui::draw(f, &mut app, &mut print))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                app.handle_input(&mut print, key);
            }
        }
        app.sync_file(&mut print);
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
