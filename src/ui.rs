use crate::app::App;
use crate::print::Print;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Tabs, Wrap},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App, print: &mut Print) {
    if app.show_help {
        draw_help(f, app);
    } else if app.show_history {
        draw_history(f, app, print);
    } else {
        let mut tab_titles = Vec::new();
        for e in &(app.tabs.tabs) {
            tab_titles.push(e.title.clone());
        }

        let titles = tab_titles
            .iter()
            .map(|t| Spans::from(Span::styled(t, Style::default().fg(Color::Green))))
            .collect();
        let tabs = Tabs::new(titles)
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(app.tabs.index);
        let rect = Rect::new(0, 0, f.size().width, 1);
        f.render_widget(tabs, rect);
        let rect = Rect::new(0, 1, f.size().width, f.size().height - 2);
        draw_tabs(f, app, rect);
    }

    if app.enter_prompt {
        let rect = Rect::new(0, f.size().height - 1, f.size().width, 1);
        let widget = app.textarea.widget();
        f.render_widget(widget, rect);
    }
}

fn draw_tabs<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    if !app.files.files.is_empty() {
        draw_tab(f, app, area);
    }
}

fn draw_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    app.tabs.tabs[app.tabs.index].print_height = area.height - 1;
    let data = app.on_draw();

    let paragraph = Paragraph::new(data.to_vec()).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_help<B>(f: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let area = Rect::new(0, 0, f.size().width, f.size().height - 1);
    let paragraph = Paragraph::new(app.get_help())
        .wrap(Wrap { trim: true })
        .scroll((0, 0));
    f.render_widget(paragraph, area);
}

fn draw_history<B>(f: &mut Frame<B>, _app: &mut App, print: &mut Print)
where
    B: Backend,
{
    let area = Rect::new(0, 0, f.size().width, f.size().height - 1);
    let mut last_history = Vec::new();
    for l in print
        .history
        .history
        .iter()
        .rev()
        .skip(print.history.scroll)
        .take(area.height as usize)
        .rev()
    {
        last_history.push(l.clone());
    }
    let paragraph = Paragraph::new(last_history)
        .wrap(Wrap { trim: true })
        .scroll((0, 0));
    f.render_widget(paragraph, area);
}
