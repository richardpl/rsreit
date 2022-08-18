use crate::modes::element_display_size;
use crate::modes::element_mode_size;
use crate::modes::AsmDisplay;
use crate::modes::Display;
use crate::modes::ElementDisplay;
use crate::modes::ElementMode;
use crate::modes::PrintDisplay;
use crate::modes::VisualDisplay;

#[derive(Clone, Eq, PartialEq)]
pub struct Tab {
    pub title: String,
    pub fileitem_index: usize,
    pub print_width: usize,
    pub print_height: u16,
    pub display: Display,
    pub element_display: ElementDisplay,
    pub print_display: PrintDisplay,
    pub element_mode: ElementMode,
    pub asm_display: AsmDisplay,
    pub visual_display: VisualDisplay,
    pub insert_mode: bool,
    pub insert_index: usize,
    pub insert_vector: [u8; 64],
    pub cursor_row: u16,
    pub cursor_column: u16,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Tabs {
    pub tabs: Vec<Tab>,
    pub index: usize,
}

impl Tabs {
    pub fn default() -> Tabs {
        Tabs {
            tabs: Vec::new(),
            index: 0,
        }
    }

    pub fn next(&mut self) {
        if !self.tabs.is_empty() {
            self.index = (self.index + 1) % self.tabs.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.tabs.is_empty() {
            if self.index > 0 {
                self.index -= 1;
            } else {
                self.index = self.tabs.len() - 1;
            }
        }
    }

    pub fn add(&mut self, title: String) {
        let new_tab = Tab {
            title,
            fileitem_index: 0,
            print_width: 16,
            print_height: 1,
            display: Display::Element,
            element_display: ElementDisplay::Byte,
            print_display: PrintDisplay::ASCIIPrint,
            element_mode: ElementMode::Hex,
            asm_display: AsmDisplay::Nasm,
            visual_display: VisualDisplay::Color,
            insert_mode: false,
            insert_index: 0,
            insert_vector: [0u8; 64],
            cursor_row: 0,
            cursor_column: 0,
        };
        self.tabs.push(new_tab);
    }

    pub fn current(&mut self) -> &mut Tab {
        let tab_index = self.index;
        &mut self.tabs[tab_index]
    }

    pub fn file_index(&mut self) -> usize {
        self.tabs[self.index].fileitem_index
    }

    pub fn cursor_pos(&mut self) -> usize {
        let tab = self.current();
        let size = element_display_size(tab.element_display);
        let print_width = tab.print_width;
        let column = tab.cursor_column & !(size - 1);
        let row = tab.cursor_row;
        print_width * row as usize + column as usize
    }

    pub fn element_input_size(tab: &mut Tab) -> u16 {
        element_display_size(tab.element_display) * element_mode_size(tab.element_mode)
    }

    pub fn element_mode(&mut self, mode: String) {
        if mode.eq("hex") {
            self.current().element_mode = ElementMode::Hex;
        } else if mode.eq("dec") {
            self.current().element_mode = ElementMode::Dec;
        } else if mode.eq("oct") {
            self.current().element_mode = ElementMode::Oct;
        } else if mode.eq("bin") {
            self.current().element_mode = ElementMode::Bin;
        }
    }

    pub fn insert_index_next(&mut self) {
        let insert_size = Self::element_input_size(self.current());
        let mut insert_index = self.current().insert_index;
        insert_index += 1;
        if insert_index >= insert_size as usize {
            insert_index = 0;
        }
        self.current().insert_index = insert_index;
    }

    pub fn cursor_left(&mut self) {
        let size = element_display_size(self.current().element_display);
        let mut column = self.current().cursor_column;
        column &= !(size - 1) as u16;
        if column >= size {
            column -= size;
        }
        self.current().cursor_column = column & !(size - 1) as u16;
    }

    pub fn cursor_right(&mut self) {
        let size = element_display_size(self.current().element_display);
        let width = self.current().print_width as u16;
        let mut column = self.current().cursor_column;
        column &= !(size - 1) as u16;
        if column < (width - size) as u16 {
            column += size;
        } else {
            column = (width - size) as u16;
        }
        self.current().cursor_column = column & !(size - 1) as u16;
    }
}
