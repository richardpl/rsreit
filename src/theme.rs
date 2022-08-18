use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;

#[derive(Copy, Clone)]
pub struct Theme {
    pub ascii: Style,
    pub current_offset: Style,
    pub current_text: Style,
    pub data: Style,
    pub decorator: Style,
    pub directive: Style,
    pub edited: Style,
    pub error: Style,
    pub function: Style,
    pub functionaddress: Style,
    pub header: Style,
    pub keyword: Style,
    pub label: Style,
    pub labeladdress: Style,
    pub mnemonic: Style,
    pub noascii: Style,
    pub null: Style,
    pub number: Style,
    pub offset: Style,
    pub prefix: Style,
    pub punctuation: Style,
    pub register: Style,
    pub selectorvalue: Style,
    pub tab: Style,
    pub text: Style,
}

impl Theme {
    pub fn default() -> Theme {
        Theme {
            current_offset: Style::default().bg(Color::Green).fg(Color::Black),
            current_text: Style::default().bg(Color::White).fg(Color::Black),
            data: Style::default().fg(Color::Yellow).bg(Color::Black),
            decorator: Style::default().fg(Color::Yellow).bg(Color::Black),
            directive: Style::default().fg(Color::Red).bg(Color::Black),
            error: Style::default().fg(Color::Black).bg(Color::Red),
            function: Style::default().fg(Color::Yellow).bg(Color::Black),
            functionaddress: Style::default().fg(Color::Yellow).bg(Color::Black),
            header: Style::default()
                .fg(Color::Green)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            keyword: Style::default().fg(Color::Cyan).bg(Color::Black),
            label: Style::default()
                .fg(Color::Rgb(0x11, 0xaa, 0x33))
                .bg(Color::Black),
            labeladdress: Style::default()
                .fg(Color::Rgb(0xb1, 0x9a, 0x13))
                .bg(Color::Black),
            mnemonic: Style::default().fg(Color::Blue).bg(Color::Black),
            number: Style::default().fg(Color::Yellow).bg(Color::Black),
            offset: Style::default().fg(Color::Green).bg(Color::Black),
            prefix: Style::default().fg(Color::Magenta).bg(Color::Black),
            punctuation: Style::default().fg(Color::Yellow).bg(Color::Black),
            register: Style::default().fg(Color::Green).bg(Color::Black),
            selectorvalue: Style::default().fg(Color::Yellow).bg(Color::Black),
            ascii: Style::default().fg(Color::Yellow).bg(Color::Black),
            noascii: Style::default().fg(Color::Red).bg(Color::Black),
            text: Style::default().fg(Color::White).bg(Color::Black),
            null: Style::default().fg(Color::Black).bg(Color::Black),
            tab: Style::default().fg(Color::Cyan).bg(Color::Black),
            edited: Style::default().fg(Color::Yellow).bg(Color::Rgb(0x20, 0x20, 0x20)),
        }
    }
}
