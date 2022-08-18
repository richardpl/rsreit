use tui::style::Style;
use tui::text::Span;
use tui::text::Spans;

#[derive(Clone)]
pub struct History<'a> {
    pub history: Vec<Spans<'a>>,
    pub scroll: usize,
}

impl<'a> History<'a> {
    pub fn default() -> History<'a> {
        History {
            history: Vec::new(),
            scroll: 0,
        }
    }

    pub fn print(&mut self, style: Style, msg: String) {
        let mut line = Vec::new();
        let mut buffer = Vec::new();
        line.push(Span::styled(msg, style));
        buffer.push(Spans::from(line));
        self.history.push(buffer[0].clone());
    }

    pub fn add(&mut self, buffer: &Vec<Spans<'a>>) {
        for l in buffer {
            self.history.push(l.clone());
        }
    }

    pub fn scroll_up(&mut self, amount: usize) {
        if self.scroll < self.history.len() {
            self.scroll += amount;
        }
        if self.scroll > self.history.len() {
            self.scroll = self.history.len() - 1;
        }
    }

    pub fn scroll_down(&mut self, amount: usize) {
        if self.scroll > amount {
            self.scroll -= amount;
        } else {
            self.scroll = 0;
        }
        if self.scroll > self.history.len() {
            self.scroll = self.history.len() - 1;
        }
    }
}
