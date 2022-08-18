use crate::app::App;
use crate::history::History;

#[derive(Clone)]
pub struct Print<'a> {
    pub history: History<'a>,
}

impl<'a> Print<'a> {
    pub fn default() -> Print<'a> {
        Print {
            history: History::default(),
        }
    }

    pub fn hexbyte(&mut self, app: &mut App<'a>) {
        let buffer = App::get_hexbyte(app);
        self.history.add(buffer);
    }

    pub fn decbyte(&mut self, app: &mut App<'a>) {
        let buffer = App::get_decbyte(app);
        self.history.add(buffer);
    }

    pub fn octbyte(&mut self, app: &mut App<'a>) {
        let buffer = App::get_octbyte(app);
        self.history.add(buffer);
    }

    pub fn binbyte(&mut self, app: &mut App<'a>) {
        let buffer = App::get_binbyte(app);
        self.history.add(buffer);
    }

    pub fn color(&mut self, app: &mut App<'a>) {
        let buffer = App::get_color(app);
        self.history.add(buffer);
    }

    pub fn hexword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_hexword(app);
        self.history.add(buffer);
    }

    pub fn decword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_decword(app);
        self.history.add(buffer);
    }

    pub fn octword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_octword(app);
        self.history.add(buffer);
    }

    pub fn binword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_binword(app);
        self.history.add(buffer);
    }

    pub fn hexdword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_hexword(app);
        self.history.add(buffer);
    }

    pub fn decdword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_decword(app);
        self.history.add(buffer);
    }

    pub fn octdword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_octword(app);
        self.history.add(buffer);
    }

    pub fn bindword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_binword(app);
        self.history.add(buffer);
    }

    pub fn hexqword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_hexqword(app);
        self.history.add(buffer);
    }

    pub fn decqword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_decqword(app);
        self.history.add(buffer);
    }

    pub fn octqword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_octqword(app);
        self.history.add(buffer);
    }

    pub fn binqword(&mut self, app: &mut App<'a>) {
        let buffer = App::get_binqword(app);
        self.history.add(buffer);
    }

    pub fn asm(&mut self, app: &mut App<'a>) {
        let buffer = App::get_asm(app);
        self.history.add(buffer);
    }

    pub fn ascii_escape(&mut self, app: &mut App<'a>) {
        let buffer = App::get_ascii_escape(app);
        self.history.add(buffer);
    }

    pub fn ascii_print(&mut self, app: &mut App<'a>) {
        let buffer = App::get_ascii_print(app);
        self.history.add(buffer);
    }

    pub fn unicode_escape(&mut self, app: &mut App<'a>) {
        let buffer = App::get_unicode_escape(app);
        self.history.add(buffer);
    }

    pub fn unicode_print(&mut self, app: &mut App<'a>) {
        let buffer = App::get_unicode_print(app);
        self.history.add(buffer);
    }

    pub fn entropy(&mut self, app: &mut App<'a>) {
        let buffer = App::get_entropy(app);
        self.history.add(buffer);
    }
}
