use crate::block::Block;
use crate::data::Data;
use crate::files::File;
use crate::files::Files;
use crate::hits::Hits;
use crate::modes::element_display_size;
use crate::modes::element_mode_base;
use crate::modes::AsmDisplay;
use crate::modes::Display;
use crate::modes::ElementDisplay;
use crate::modes::ElementMode;
use crate::modes::PrintDisplay;
use crate::modes::VisualDisplay;
use crate::print::Print;
use crate::tabs::Tabs;
use crate::theme::Theme;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use memmem::{Searcher, TwoWaySearcher};
use safe_transmute::base::from_bytes;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::Cursor;
use std::iter::Iterator;
use std::mem::size_of;
use std::num::ParseIntError;
use std::result::Result;
use std::time::Instant;

use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};

use tui_textarea::TextArea;

use iced_x86::{
    Decoder, DecoderOptions, FormatterOutput, FormatterTextKind, GasFormatter, Instruction,
    IntelFormatter, MasmFormatter, NasmFormatter,
};

pub const DISPLAYS: &[Display] = &[
    Display::Element,
    Display::Asm,
    Display::Print,
    Display::Visual,
];

pub const ELEMENT_DISPLAYS: &[ElementDisplay] = &[
    ElementDisplay::Byte,
    ElementDisplay::Word,
    ElementDisplay::DWord,
    ElementDisplay::QWord,
];

pub const PRINT_DISPLAYS: &[PrintDisplay] = &[
    PrintDisplay::ASCIIPrint,
    PrintDisplay::ASCIIEscape,
    PrintDisplay::UnicodePrint,
    PrintDisplay::UnicodeEscape,
];

pub const VISUAL_DISPLAYS: &[VisualDisplay] = &[VisualDisplay::Color, VisualDisplay::Entropy];

pub const ELEMENT_MODES: &[ElementMode] = &[
    ElementMode::Hex,
    ElementMode::Dec,
    ElementMode::Oct,
    ElementMode::Bin,
];

pub const ASM_DISPLAYS: &[AsmDisplay] = &[
    AsmDisplay::Nasm,
    AsmDisplay::Masm,
    AsmDisplay::Gas,
    AsmDisplay::Intel,
];

struct AsmFormatterOutput {
    pub vec: Vec<(String, FormatterTextKind)>,
}

impl AsmFormatterOutput {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }
}

impl FormatterOutput for AsmFormatterOutput {
    fn write(&mut self, text: &str, kind: FormatterTextKind) {
        self.vec.push((String::from(text), kind));
    }
}

const HEXBYTES_COLUMN_BYTE_LENGTH: usize = 16;

#[derive(Clone)]
pub struct Cache<'a> {
    pub buffer: Vec<Spans<'a>>,
}

impl<'a> Cache<'a> {
    pub fn default() -> Cache<'a> {
        Cache { buffer: Vec::new() }
    }
}

pub struct App<'a> {
    pub title: &'a str,
    pub paths: Vec<String>,
    pub should_quit: bool,
    pub enter_prompt: bool,
    pub show_history: bool,
    pub show_help: bool,
    pub files: Files,
    pub tabs: Tabs,
    pub progress: f64,
    pub now: Instant,
    pub textarea: TextArea<'a>,
    pub cache: Cache<'a>,
    pub theme: Theme,
    pub nasm_formatter: NasmFormatter,
    pub masm_formatter: MasmFormatter,
    pub gas_formatter: GasFormatter,
    pub intel_formatter: IntelFormatter,
}

macro_rules! get_header {
    ($hdr_fmt:literal, $idx:ident) => {
        if $idx == 0 {
            format!("  -offset-   ")
        } else {
            format!($hdr_fmt, ($idx - 1) & 15)
        }
    };
}

macro_rules! get_ascii {
    ($z:ident, $theme:ident) => {
        if $z.is_ascii_graphic() {
            Span::styled(format!("{}", $z as char), $theme.ascii)
        } else {
            Span::styled(format!("{}", '.'), $theme.noascii)
        }
    };
}

macro_rules! get_values {
    ($element_type:ty, $fmt:literal, $reader:ident, $ivector:ident, $pw:ident, $x:ident, $y:ident, $column:ident, $row:ident, $offset:ident, $buffer:ident, $theme:ident, $source:ident) => {
        if $x == 0 {
            if $reader.position() >= $buffer.len() as u64 {
                Span::styled(" ", $theme.null)
            } else if $row == $y {
                const ELEMENT_SIZE: usize = size_of::<$element_type>() as usize;
                Span::styled(
                    format!(
                        "0x{:08x} ",
                        $offset + (ELEMENT_SIZE * $pw * $y as usize) as u64
                    ),
                    $theme.current_offset,
                )
            } else {
                const ELEMENT_SIZE: usize = size_of::<$element_type>() as usize;
                Span::styled(
                    format!(
                        "0x{:08x} ",
                        $offset + (ELEMENT_SIZE * $pw * $y as usize) as u64
                    ),
                    $theme.offset,
                )
            }
        } else if $x == $pw + 1 {
            Span::styled("  ", $theme.null)
        } else if $x > $pw + 1 {
            if $reader.position() >= $buffer.len() as u64 {
                Span::styled(" ", $theme.null)
            } else {
                const ELEMENT_SIZE: usize = size_of::<$element_type>() as usize;
                let idx = ELEMENT_SIZE * ($pw * $y as usize) + $x - $pw - 2;
                if idx < $buffer.len() {
                    let c = $buffer[idx];
                    get_ascii!(c, $theme)
                } else {
                    Span::styled(" ", $theme.null)
                }
            }
        } else {
            if $reader.position() >= $buffer.len() as u64 {
                Span::styled(" ", $theme.null)
            } else {
                const ELEMENT_SIZE: usize = size_of::<$element_type>() as usize;
                let mut ovector: [u8; ELEMENT_SIZE] = [0; ELEMENT_SIZE];
                let mut vector: [u8; ELEMENT_SIZE] = [0; ELEMENT_SIZE];
                let _read = $reader.read(&mut vector);
                let original;
                let val;
                unsafe {
                    val = from_bytes::<$element_type>(&vector);
                }
                let _oread = $source.read(&mut ovector);
                unsafe {
                    original = from_bytes::<$element_type>(&ovector);
                }
                let style;
                if val != original {
                    style = $theme.edited;
                } else {
                    style = $theme.text;
                }
                if $row == $y
                    && usize::from($column) >= ($x - 1) * ELEMENT_SIZE
                    && usize::from($column) < ($x) * ELEMENT_SIZE
                {
                    let number = val.unwrap();
                    let zz = format!($fmt, number);
                    let letters: Vec<u8> = zz.trim().as_bytes().to_vec();
                    *$ivector = Self::pop(&letters);
                    if usize::from($column) == ($x - 1) * ELEMENT_SIZE {
                        Span::styled(zz, $theme.current_text)
                    } else {
                        Span::styled(zz, style)
                    }
                } else {
                    Span::styled(format!($fmt, val.unwrap()), style)
                }
            }
        }
    };
}

macro_rules! get_element {
    ($element_type:ty, $app:ident, $fmt:literal, $hdr_fmt:literal) => {
        let cache = &mut $app.cache;
        let theme = $app.theme;
        let fi = $app.files.current($app.tabs.file_index());
        let ti = $app.tabs.current();
        const ELEMENT_SIZE: usize = size_of::<$element_type>() as usize;
        let print_width = std::cmp::max((ti.print_width + (ELEMENT_SIZE - 1)) / ELEMENT_SIZE, 1);
        let print_height = ti.print_height;
        let mut row = ti.cursor_row;
        let column = ti.cursor_column & !((ELEMENT_SIZE - 1) as u16);
        let ivector = &mut ti.insert_vector;
        let buffer = &fi.block.buffer;
        let mut source = Cursor::new(&fi.block.source);
        let mut reader = Cursor::new(&fi.block.buffer);
        let offset = fi.block.offset;

        if !ti.insert_mode {
            row = print_height + 1;
        }

        cache.buffer.clear();

        cache.buffer.push(tui::text::Spans(
            (0..print_width + 1)
                .map(|x| Span::styled(get_header!($hdr_fmt, x), theme.header))
                .collect::<Vec<Span>>(),
        ));

        for y in 0..print_height {
            cache.buffer.push(tui::text::Spans(
                (0..(print_width + print_width * ELEMENT_SIZE) + 2)
                    .map(|x| {
                        get_values!(
                            $element_type,
                            $fmt,
                            reader,
                            ivector,
                            print_width,
                            x,
                            y,
                            column,
                            row,
                            offset,
                            buffer,
                            theme,
                            source
                        )
                    })
                    .collect::<Vec<Span>>(),
            ));
        }
    };
}

macro_rules! flush_input_item {
    ($element_type:ty, $size:ident, $input:ident, $base:ident, $vv:ident, $r:ident) => {
        let s = String::from_utf8($input.get(0..$size as usize).unwrap().to_vec()).unwrap();
        let v = <$element_type>::from_str_radix(&s, $base);
        if v.is_ok() {
            *$vv = v.unwrap().to_le_bytes().to_vec();
            $r = true;
        }
    };
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, paths: Vec<String>) -> App<'a> {
        App {
            title,
            paths,
            should_quit: false,
            enter_prompt: false,
            show_history: false,
            show_help: false,
            progress: 0.0,
            now: Instant::now(),
            textarea: TextArea::default(),
            cache: Cache::default(),
            files: Files::default(),
            tabs: Tabs::default(),
            theme: Theme::default(),
            nasm_formatter: NasmFormatter::new(),
            masm_formatter: MasmFormatter::new(),
            gas_formatter: GasFormatter::new(),
            intel_formatter: IntelFormatter::new(),
        }
    }

    fn handle_search(&mut self, item: String) -> io::Result<usize> {
        let path = &self.files.current_path(&mut self.tabs);
        let mut file = std::fs::File::open(path)?;
        let len = fs::metadata(path)?.len();
        let mut block = Block::new(2048usize);
        let mut offset = 0u64;
        let search_bytes = item.as_str().as_bytes();
        let search_len = search_bytes.len() as u64;
        let search = TwoWaySearcher::new(search_bytes);
        let mut hits = Hits::new(item.clone());

        while offset < len {
            block.offset = offset;
            Files::read_block(
                &mut file,
                block.size + search_len - 1,
                block.offset,
                len,
                &mut block.buffer,
            )?;
            let r = search.search_in(&block.buffer);
            if r.is_some() {
                let hit_offset = offset + r.unwrap() as u64;
                hits.hits.push(hit_offset);
            }
            offset += block.size;
        }
        let fi = &mut self.files.current(self.tabs.file_index());
        let found_items = hits.hits.len();
        fi.hhits.add(hits);
        Ok(found_items)
    }

    fn handle_print(&mut self, print: &mut Print<'a>, kind: String, mode: String) {
        if !self.files.files.is_empty() {
            if kind.eq("byte") {
                if mode.eq("hex") {
                    print.hexbyte(self);
                } else if mode.eq("dec") {
                    print.decbyte(self);
                } else if mode.eq("oct") {
                    print.octbyte(self);
                } else if mode.eq("bin") {
                    print.binbyte(self);
                }
            } else if kind.eq("word") {
                if mode.eq("hex") {
                    print.hexword(self);
                } else if mode.eq("dec") {
                    print.decword(self);
                } else if mode.eq("oct") {
                    print.octword(self);
                } else if mode.eq("bin") {
                    print.binword(self);
                }
            } else if kind.eq("dword") {
                if mode.eq("hex") {
                    print.hexdword(self);
                } else if mode.eq("dec") {
                    print.decdword(self);
                } else if mode.eq("oct") {
                    print.octdword(self);
                } else if mode.eq("bin") {
                    print.bindword(self);
                }
            } else if kind.eq("qword") {
                if mode.eq("hex") {
                    print.hexqword(self);
                } else if mode.eq("dec") {
                    print.decqword(self);
                } else if mode.eq("oct") {
                    print.octqword(self);
                } else if mode.eq("bin") {
                    print.binqword(self);
                }
            } else if kind.eq("asm") {
                print.asm(self);
            } else if kind.eq("print") {
                if mode.eq("ascii") {
                    print.ascii_print(self);
                } else if mode.eq("ascii_escape") {
                    print.ascii_escape(self);
                } else if mode.eq("unicode") {
                    print.unicode_print(self);
                } else if mode.eq("unicode_escape") {
                    print.unicode_escape(self);
                }
            } else if kind.eq("visual") {
                if mode.eq("color") {
                    print.color(self);
                } else if mode.eq("entropy") {
                    print.entropy(self);
                }
            }
        }
    }

    fn handle_show(&mut self, kind: String, mode: String) {
        if !self.files.files.is_empty() {
            if kind.eq("byte") {
                self.tabs.current().display = Display::Element;
                self.tabs.current().element_display = ElementDisplay::Byte;
                self.tabs.element_mode(mode);
            } else if kind.eq("word") {
                self.tabs.current().display = Display::Element;
                self.tabs.current().element_display = ElementDisplay::Word;
                self.tabs.element_mode(mode);
            } else if kind.eq("dword") {
                self.tabs.current().display = Display::Element;
                self.tabs.current().element_display = ElementDisplay::DWord;
                self.tabs.element_mode(mode);
            } else if kind.eq("qword") {
                self.tabs.current().display = Display::Element;
                self.tabs.current().element_display = ElementDisplay::QWord;
                self.tabs.element_mode(mode);
            } else if kind.eq("asm") {
                self.tabs.current().display = Display::Asm;
            } else if kind.eq("print") {
                self.tabs.current().display = Display::Print;
            } else if kind.eq("visual") {
                self.tabs.current().display = Display::Visual;
            }
        }
    }

    pub fn get_decbyte(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u8, self, " {:^03}", " {:^3x}");
        &self.cache.buffer
    }

    pub fn get_octbyte(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u8, self, " {:<03o}", " {:^3x}");
        &self.cache.buffer
    }

    pub fn get_binbyte(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u8, self, " {:<08b}", " {:^8x}");
        &self.cache.buffer
    }

    pub fn get_hexbyte(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u8, self, " {:<02x}", " {:^2x}");
        &self.cache.buffer
    }

    pub fn get_color(&mut self) -> &Vec<Spans<'a>> {
        let offset_style = self.theme.offset;
        let print_width = self.tabs.current().print_width;
        let fi = self.files.current(self.tabs.file_index());
        let buffer = &mut self.cache.buffer;
        let mut line = Vec::new();
        let hex_iter = fi.block.buffer.iter();
        let mut offset = fi.block.offset;
        let mut i = 0;

        buffer.clear();
        for val in hex_iter {
            let hex_val = String::from("__");
            let red = (*val as u8).rotate_left(4);
            let blue = (*val as u8).rotate_right(2);
            let green = *val as u8;
            let hex_color = Style::default()
                .fg(Color::Rgb(red, green, blue))
                .bg(Color::Rgb(red, green, blue));
            if i == 0 {
                line.push(Span::styled(format!("0x{:08x} ", offset), offset_style));
            }
            line.push(Span::styled(hex_val, hex_color));

            i += 1;
            if i >= print_width as u64 {
                i = 0;
                offset += print_width as u64;
            }
            if i == 0 {
                buffer.push(Spans::from(line.clone()));
                line.clear();
            }
        }
        &self.cache.buffer
    }

    fn calc_entropy(block: &Block) -> f64 {
        let mut histogram = [0u64; 256];
        let hex_iter = block.buffer.iter();
        for val in hex_iter {
            histogram[*val as usize] += 1u64;
        }
        let mut entropy: f64 = 0.0;
        let scale: f64 = 1.0f64 / (block.size as f64);
        for i in 0..256 {
            if histogram[i] > 0u64 {
                let p: f64 = histogram[i] as f64 * scale;
                entropy += p * -p.log2();
            }
        }
        entropy
    }

    pub fn get_entropy(&mut self) -> &Vec<Spans<'a>> {
        let path = &self.files.current_path(&mut self.tabs);
        let mut file = std::fs::File::open(path).unwrap();
        let len = fs::metadata(path).expect("bug").len();
        let print_width = self.tabs.current().print_width;
        let print_height = self.tabs.current().print_height;
        let fi = self.files.current(self.tabs.file_index());
        let buffer = &mut self.cache.buffer;
        let mut offset = fi.block.offset;
        let mut block = Block::new(2048usize);

        buffer.clear();
        for _i in 0..print_height {
            if offset >= len {
                break;
            }
            block.offset = offset;
            let r = Files::read_block(&mut file, block.size, block.offset, len, &mut block.buffer);
            if r.is_err() {
                break;
            }
            let entropy = Self::calc_entropy(&block);
            let scaled = ((255.0f64 * entropy).round()) as u8;
            let width = (entropy * (print_width as f64)).round() as u64;
            let red = scaled.rotate_left(4);
            let blue = scaled.rotate_right(2);
            let green = scaled;
            let hex_color = Style::default()
                .fg(Color::Rgb(red, green, blue))
                .bg(Color::Rgb(red, green, blue));

            buffer.push(tui::text::Spans(
                (0..width)
                    .map(|_x| Span::styled("_", hex_color))
                    .collect::<Vec<Span>>(),
            ));
            offset += block.size;
        }
        &self.cache.buffer
    }

    pub fn get_decword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u16, self, " {:^05}", " {:^5x}");
        &self.cache.buffer
    }

    pub fn get_octword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u16, self, " {:06o}", " {:^6x}");
        &self.cache.buffer
    }

    pub fn get_binword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u16, self, " {:016b}", " {:^16x}");
        &self.cache.buffer
    }

    pub fn get_hexword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u16, self, " {:04x}", " {:^4x}");
        &self.cache.buffer
    }

    pub fn get_decdword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u32, self, " {:^010}", " {:^10x}");
        &self.cache.buffer
    }

    pub fn get_octdword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u32, self, " {:011o}", " {:^11x}");
        &self.cache.buffer
    }

    pub fn get_bindword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u32, self, " {:032b}", " {:^32x}");
        &self.cache.buffer
    }

    pub fn get_hexdword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u32, self, " {:08x}", " {:^8x}");
        &self.cache.buffer
    }

    pub fn get_decqword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u64, self, " {:^020}", " {:^20x}");
        &self.cache.buffer
    }

    pub fn get_octqword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u64, self, " {:022o}", " {:^22x}");
        &self.cache.buffer
    }

    pub fn get_binqword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u64, self, " {:064b}", " {:^64x}");
        &self.cache.buffer
    }

    pub fn get_hexqword(&mut self) -> &Vec<Spans<'a>> {
        get_element!(u64, self, " {:016x}", " {:^16x}");
        &self.cache.buffer
    }

    fn get_asm_fmt<T: iced_x86::Formatter>(
        fi: &File,
        cache: &mut Cache,
        theme: Theme,
        formatter: &mut T,
    ) {
        let mut theme = theme;
        let buffer = &mut cache.buffer;
        let mut line = Vec::new();
        let current_offset = fi.block.offset;
        let bytes = &fi.block.buffer;
        let mut decoder = Decoder::with_ip(64, bytes, current_offset, DecoderOptions::NONE);

        buffer.clear();

        // Change some options, there are many more
        formatter.options_mut().set_digit_separator("`");
        formatter.options_mut().set_first_operand_char_index(10);

        // Initialize this outside the loop because decode_out() writes to every field
        let mut instruction = Instruction::default();

        // The decoder also implements Iterator/IntoIterator so you could use a for loop:
        //      for instruction in &mut decoder { /* ... */ }
        // or collect():
        //      let instructions: Vec<_> = decoder.into_iter().collect();
        // but can_decode()/decode_out() is a little faster:
        while decoder.can_decode() {
            // There's also a decode() method that returns an instruction but that also
            // means it copies an instruction (40 bytes):
            //     instruction = decoder.decode();
            decoder.decode_out(&mut instruction);

            line.push(Span::styled(
                format!("{:016X} ", instruction.ip()),
                theme.offset,
            ));
            let start_index = (instruction.ip() - current_offset) as usize;
            let instr_bytes = &bytes[start_index..start_index + instruction.len()];
            for b in instr_bytes.iter() {
                line.push(Span::styled(format!("{:02X}", b), theme.header));
            }
            if instr_bytes.len() < HEXBYTES_COLUMN_BYTE_LENGTH {
                for _ in 0..HEXBYTES_COLUMN_BYTE_LENGTH - instr_bytes.len() {
                    line.push(Span::styled("  ", theme.text));
                }
            }
            let mut output = AsmFormatterOutput::new();
            formatter.format(&instruction, &mut output);
            for (text, kind) in output.vec.iter() {
                line.push(Span::styled(
                    text.clone(),
                    Self::get_asm_color(*kind, &mut theme),
                ));
            }
            buffer.push(Spans::from(line));
            line = Vec::new();
        }
    }

    pub fn get_asm(&mut self) -> &Vec<Spans<'a>> {
        let cache = &mut self.cache;
        let theme = self.theme;
        let file_index = self.tabs.tabs[self.tabs.index].fileitem_index;
        let asm_display = self.tabs.tabs[self.tabs.index].asm_display;
        let fi = &self.files.files[file_index];
        match asm_display {
            AsmDisplay::Nasm => Self::get_asm_fmt(fi, cache, theme, &mut self.nasm_formatter),
            AsmDisplay::Masm => Self::get_asm_fmt(fi, cache, theme, &mut self.masm_formatter),
            AsmDisplay::Gas => Self::get_asm_fmt(fi, cache, theme, &mut self.gas_formatter),
            AsmDisplay::Intel => Self::get_asm_fmt(fi, cache, theme, &mut self.intel_formatter),
        }
        &self.cache.buffer
    }

    fn get_asm_color(kind: FormatterTextKind, theme: &mut Theme) -> Style {
        match kind {
            FormatterTextKind::Data => theme.data,
            FormatterTextKind::Decorator => theme.decorator,
            FormatterTextKind::Directive => theme.directive,
            FormatterTextKind::Function => theme.function,
            FormatterTextKind::FunctionAddress => theme.functionaddress,
            FormatterTextKind::Keyword => theme.keyword,
            FormatterTextKind::Label => theme.label,
            FormatterTextKind::LabelAddress => theme.labeladdress,
            FormatterTextKind::Mnemonic => theme.mnemonic,
            FormatterTextKind::Number => theme.number,
            FormatterTextKind::Prefix => theme.prefix,
            FormatterTextKind::Punctuation => theme.punctuation,
            FormatterTextKind::Register => theme.register,
            FormatterTextKind::SelectorValue => theme.selectorvalue,
            _ => theme.text,
        }
    }

    pub fn get_ascii_print(&mut self) -> &Vec<Spans<'a>> {
        let theme = self.theme;
        let fi = self.files.current(self.tabs.file_index());
        let buffer = &mut self.cache.buffer;
        let mut line = Vec::new();
        let iter = fi.block.buffer.iter();

        buffer.clear();
        for val in iter {
            if *val == b'\n' {
                buffer.push(Spans::from(line.clone()));
                line.clear();
            } else if *val == b'\t' {
                line.push(Span::styled("..", theme.tab));
            } else if val.is_ascii_graphic() || val.is_ascii_whitespace() {
                line.push(Span::styled(format!("{}", *val as char), theme.text));
            } else {
                line.push(Span::styled(" ", theme.text));
            }
        }
        buffer.push(Spans::from(line));
        &self.cache.buffer
    }

    pub fn get_ascii_escape(&mut self) -> &Vec<Spans<'a>> {
        let theme = self.theme;
        let fi = self.files.current(self.tabs.file_index());
        let buffer = &mut self.cache.buffer;
        let mut line = Vec::new();
        let iter = fi.block.buffer.iter();

        buffer.clear();
        for val in iter {
            let c = *val as char;
            line.push(Span::styled(c.escape_default().to_string(), theme.text));
            if c == '\n' {
                buffer.push(Spans::from(line.clone()));
                line.clear();
            }
        }
        buffer.push(Spans::from(line));
        &self.cache.buffer
    }

    pub fn get_unicode_print(&mut self) -> &Vec<Spans<'a>> {
        let theme = self.theme;
        let fi = self.files.current(self.tabs.file_index());
        let buffer = &mut self.cache.buffer;
        let mut line = Vec::new();
        let iter = fi.block.buffer.iter();

        buffer.clear();
        for val in iter {
            let c = *val as char;
            line.push(Span::styled(format!("{}", c), theme.text));
            if c == '\n' {
                buffer.push(Spans::from(line.clone()));
                line.clear();
            }
        }
        buffer.push(Spans::from(line));
        &self.cache.buffer
    }

    pub fn get_unicode_escape(&mut self) -> &Vec<Spans<'a>> {
        let theme = self.theme;
        let fi = self.files.current(self.tabs.file_index());
        let buffer = &mut self.cache.buffer;
        let mut line = Vec::new();
        let iter = fi.block.buffer.iter();

        buffer.clear();
        for val in iter {
            let c = *val as char;
            line.push(Span::styled(c.escape_unicode().to_string(), theme.text));
            if c == '\n' {
                buffer.push(Spans::from(line.clone()));
                line.clear();
            }
        }
        buffer.push(Spans::from(line));
        &self.cache.buffer
    }

    fn set_block_size(&mut self, ret: Result<u64, ParseIntError>) {
        if !self.files.files.is_empty() && ret.is_ok() {
            let mut fi = self.files.current(self.tabs.file_index());
            let size = ret.unwrap();
            if size > 0 {
                fi.block.size = size;
            }
        }
    }

    fn set_block_offset(&mut self, ret: Result<u64, ParseIntError>) {
        if !self.files.files.is_empty() && ret.is_ok() {
            let mut fi = self.files.current(self.tabs.file_index());
            fi.block.offset = ret.unwrap();
        }
    }

    fn need_block(&mut self) -> bool {
        if !self.files.files.is_empty() {
            let fi = self.files.current(self.tabs.file_index());
            fi.block.offset != fi.block.prev_offset || fi.block.size != fi.block.prev_size
        } else {
            false
        }
    }

    fn read_block(&mut self) -> io::Result<()> {
        let path = &self.files.current_path(&mut self.tabs);
        let mut file = std::fs::File::open(path)?;
        let len = fs::metadata(path)?.len();
        let mut fi = self.files.current(self.tabs.file_index());
        Files::read_block(
            &mut file,
            fi.block.size,
            fi.block.offset,
            len,
            &mut fi.block.buffer,
        )?;
        fi.block.source.clone_from(&fi.block.buffer);
        fi.block.prev_offset = fi.block.offset;
        fi.block.prev_size = fi.block.size;
        fi.size = len;
        Ok(())
    }

    fn is_insert_mode(&mut self) -> bool {
        !self.tabs.tabs.is_empty() && self.tabs.current().insert_mode
    }

    fn on_up(&mut self, print: &mut Print) {
        if self.is_insert_mode() {
            if self.tabs.current().cursor_row > 0 {
                self.tabs.current().cursor_row -= 1;
            }
            self.tabs.current().insert_index = 0;
        } else if self.show_history {
            print.history.scroll_up(1);
        } else if !self.files.files.is_empty() {
            let pw = self.tabs.current().print_width;
            let mut fi = self.files.current(self.tabs.file_index());
            if fi.block.offset >= pw as u64 {
                fi.block.offset -= pw as u64;
            } else {
                fi.block.offset = 0u64;
            }
        }
    }

    fn on_down(&mut self, print: &mut Print) {
        if self.is_insert_mode() {
            if self.tabs.current().cursor_row < self.tabs.current().print_height - 1 {
                self.tabs.current().cursor_row += 1;
            }
            self.tabs.current().insert_index = 0;
        } else if self.show_history {
            print.history.scroll_down(1);
        } else if !self.files.files.is_empty() {
            let pw = self.tabs.current().print_width;
            let mut fi = self.files.current(self.tabs.file_index());
            if fi.block.offset < u64::MAX - pw as u64 {
                fi.block.offset += pw as u64;
            } else {
                fi.block.offset = u64::MAX;
            }
        }
    }

    fn on_pageup(&mut self, print: &mut Print) {
        if self.is_insert_mode() {
            self.tabs.current().insert_index = 0;
        } else if self.show_history {
            print.history.scroll_up(20);
        } else if !self.files.files.is_empty() {
            let pw = self.tabs.current().print_width;
            let ph = self.tabs.current().print_height;
            let mut fi = self.files.current(self.tabs.file_index());
            let size = (pw as u64) * (ph as u64);
            if fi.block.offset >= size {
                fi.block.offset -= size;
            } else {
                fi.block.offset = 0u64;
            }
        }
    }

    fn on_pagedown(&mut self, print: &mut Print) {
        if self.is_insert_mode() {
            self.tabs.current().insert_index = 0;
        } else if self.show_history {
            print.history.scroll_down(20);
        } else if !self.files.files.is_empty() {
            let pw = self.tabs.current().print_width;
            let ph = self.tabs.current().print_height;
            let mut fi = self.files.current(self.tabs.file_index());
            let size = (pw as u64) * (ph as u64);
            if fi.block.offset < u64::MAX - size {
                fi.block.offset += size;
            } else {
                fi.block.offset = u64::MAX;
            }
        }
    }

    fn on_right(&mut self, _print: &mut Print) {
        if self.is_insert_mode() {
            self.tabs.cursor_right();
            self.tabs.current().insert_index = 0;
        } else {
            self.tabs.next();
        }
    }

    fn on_left(&mut self, _print: &mut Print) {
        if self.is_insert_mode() {
            self.tabs.cursor_left();
            self.tabs.current().insert_index = 0;
        } else {
            self.tabs.previous();
        }
    }

    fn on_f1(&mut self, _print: &mut Print) {
        self.show_help = !self.show_help;
    }

    fn on_tab(&mut self, _print: &mut Print) {
        self.show_history = !self.show_history;
    }

    fn on_end(&mut self, _print: &mut Print) {
        if self.is_insert_mode() {
            let mut ti = &mut self.tabs.tabs[self.tabs.index];
            ti.cursor_column = (ti.print_width - 1) as u16;
            ti.cursor_row = ti.print_height - 1;
        } else if !self.files.files.is_empty() && !self.tabs.tabs.is_empty() {
            let mut fi = self.files.current(self.tabs.file_index());
            fi.block.offset = fi.size;
        }
    }

    fn on_home(&mut self, _print: &mut Print) {
        if self.is_insert_mode() {
            let mut ti = &mut self.tabs.tabs[self.tabs.index];
            ti.cursor_column = 0;
            ti.cursor_row = 0;
        } else if !self.files.files.is_empty() && !self.tabs.tabs.is_empty() {
            self.files.current(self.tabs.file_index()).block.offset = 0;
        }
    }

    fn on_insert(&mut self, _print: &mut Print) {
        if !self.tabs.tabs.is_empty() {
            if self.tabs.current().display == Display::Element {
                self.tabs.current().insert_mode = !self.tabs.current().insert_mode;
            }
        }
    }

    fn decrease_print_width(&mut self) {
        if !self.tabs.tabs.is_empty() {
            let mut ti = &mut self.tabs.tabs[self.tabs.index];
            let size = element_display_size(ti.element_display) as usize;
            ti.print_width &= !(size - 1);
            if ti.print_width > size {
                ti.print_width -= size;
                ti.cursor_column = std::cmp::min(ti.cursor_column, (ti.print_width - 1) as u16);
                ti.cursor_column &= !((size - 1) as u16);
            }
        }
    }

    fn increase_print_width(&mut self) {
        if !self.tabs.tabs.is_empty() {
            let mut ti = &mut self.tabs.tabs[self.tabs.index];
            let size = element_display_size(ti.element_display) as usize;
            ti.print_width &= !(size - 1);
            if ti.print_width < 65535 - size {
                ti.print_width += size;
            }
        }
    }

    fn next_display(&mut self) {
        if !self.tabs.tabs.is_empty() {
            let mut ti = &mut self.tabs.tabs[self.tabs.index];
            ti.display =
                DISPLAYS[(DISPLAYS[ti.display as usize] as usize + 1).rem_euclid(DISPLAYS.len())];
        }
    }

    fn prev_display(&mut self) {
        if !self.tabs.tabs.is_empty() {
            let mut ti = &mut self.tabs.tabs[self.tabs.index];
            ti.display = DISPLAYS[(DISPLAYS[ti.display as usize] as usize + DISPLAYS.len() - 1)
                .rem_euclid(DISPLAYS.len())];
        }
    }

    fn next_element(&mut self) {
        if !self.tabs.tabs.is_empty() {
            let mut ti = &mut self.tabs.tabs[self.tabs.index];
            if ti.display == Display::Element {
                ti.element_display = ELEMENT_DISPLAYS
                    [(ELEMENT_DISPLAYS[ti.element_display as usize] as usize + 1)
                        .rem_euclid(ELEMENT_DISPLAYS.len())];
            } else if ti.display == Display::Asm {
                ti.asm_display = ASM_DISPLAYS[(ASM_DISPLAYS[ti.asm_display as usize] as usize + 1)
                    .rem_euclid(ASM_DISPLAYS.len())];
            } else if ti.display == Display::Print {
                ti.print_display =
                    PRINT_DISPLAYS[(PRINT_DISPLAYS[ti.print_display as usize] as usize + 1)
                        .rem_euclid(PRINT_DISPLAYS.len())];
            } else if ti.display == Display::Visual {
                ti.visual_display =
                    VISUAL_DISPLAYS[(VISUAL_DISPLAYS[ti.visual_display as usize] as usize + 1)
                        .rem_euclid(VISUAL_DISPLAYS.len())];
            }
        }
    }

    fn prev_element(&mut self) {
        if !self.tabs.tabs.is_empty() {
            let mut ti = &mut self.tabs.tabs[self.tabs.index];
            if ti.display == Display::Element {
                ti.element_display = ELEMENT_DISPLAYS
                    [(ELEMENT_DISPLAYS[ti.element_display as usize] as usize
                        + ELEMENT_DISPLAYS.len()
                        - 1)
                    .rem_euclid(ELEMENT_DISPLAYS.len())];
            } else if ti.display == Display::Asm {
                ti.asm_display = ASM_DISPLAYS[(ASM_DISPLAYS[ti.asm_display as usize] as usize
                    + ASM_DISPLAYS.len()
                    - 1)
                .rem_euclid(ASM_DISPLAYS.len())];
            } else if ti.display == Display::Print {
                ti.print_display = PRINT_DISPLAYS[(PRINT_DISPLAYS[ti.print_display as usize]
                    as usize
                    + PRINT_DISPLAYS.len()
                    - 1)
                .rem_euclid(PRINT_DISPLAYS.len())];
            } else if ti.display == Display::Visual {
                ti.visual_display = VISUAL_DISPLAYS[(VISUAL_DISPLAYS[ti.visual_display as usize]
                    as usize
                    + VISUAL_DISPLAYS.len()
                    - 1)
                .rem_euclid(VISUAL_DISPLAYS.len())];
            }
        }
    }

    fn next_mode(&mut self) {
        if !self.tabs.tabs.is_empty() {
            let mut ti = &mut self.tabs.tabs[self.tabs.index];
            if ti.display == Display::Element {
                ti.element_mode = ELEMENT_MODES[(ELEMENT_MODES[ti.element_mode as usize] as usize
                    + 1)
                .rem_euclid(ELEMENT_MODES.len())];
            }
        }
    }

    fn prev_mode(&mut self) {
        if !self.tabs.tabs.is_empty() {
            let mut ti = &mut self.tabs.tabs[self.tabs.index];
            if ti.display == Display::Element {
                ti.element_mode = ELEMENT_MODES[(ELEMENT_MODES[ti.element_mode as usize] as usize
                    + ELEMENT_MODES.len()
                    - 1)
                .rem_euclid(ELEMENT_MODES.len())];
            }
        }
    }

    fn next_hit(&mut self, modifier: KeyModifiers) {
        if !self.tabs.tabs.is_empty() && !self.files.files.is_empty() {
            let fi = &mut self.files.current(self.tabs.file_index());
            if modifier != KeyModifiers::CONTROL {
                let hits = &mut fi.hhits.hits[fi.hhits.selected];
                if !hits.is_empty() {
                    hits.selected = (hits.selected + 1) % hits.hits.len();
                    fi.block.offset = hits.hits[hits.selected];
                }
            } else if !fi.hhits.hits.is_empty() {
                fi.hhits.selected = (fi.hhits.selected + 1) % fi.hhits.hits.len();
                let hits = &mut fi.hhits.hits[fi.hhits.selected];
                if !hits.is_empty() {
                    fi.block.offset = hits.hits[hits.selected];
                }
            }
        }
    }

    fn prev_hit(&mut self, modifier: KeyModifiers) {
        if !self.tabs.tabs.is_empty() && !self.files.files.is_empty() {
            let fi = &mut self.files.current(self.tabs.file_index());
            if modifier != KeyModifiers::CONTROL {
                let hits = &mut fi.hhits.hits[fi.hhits.selected];
                if !hits.is_empty() {
                    if hits.selected > 0 {
                        hits.selected -= 1;
                    } else {
                        hits.selected = hits.hits.len() - 1;
                    }
                    fi.block.offset = hits.hits[hits.selected];
                }
            } else if !fi.hhits.hits.is_empty() {
                if fi.hhits.selected > 0 {
                    fi.hhits.selected -= 1;
                } else {
                    fi.hhits.selected = fi.hhits.hits.len() - 1;
                }
                let hits = &mut fi.hhits.hits[fi.hhits.selected];
                if !hits.is_empty() {
                    fi.block.offset = hits.hits[hits.selected];
                }
            }
        }
    }

    fn do_flush_input(
        input: [u8; 64],
        size: u16,
        display_size: u16,
        base: u32,
        vv: &mut Vec<u8>,
    ) -> bool {
        let mut r = false;
        match display_size {
            1 => {
                flush_input_item!(u8, size, input, base, vv, r);
            }
            2 => {
                flush_input_item!(u16, size, input, base, vv, r);
            }
            4 => {
                flush_input_item!(u32, size, input, base, vv, r);
            }
            8 => {
                flush_input_item!(u64, size, input, base, vv, r);
            }
            _ => {}
        }
        r
    }

    fn do_update_patch(patch: &mut BTreeMap<u64, Vec<u8>>, offset: u64, value: Vec<u8>) {
        patch.insert(offset, value);
    }

    fn handle_insert(&mut self, c: char) {
        let mut vv: Vec<u8> = Vec::new();
        let tabs = &mut self.tabs;
        let pos = tabs.cursor_pos();
        let index = tabs.file_index();
        let ti = tabs.current();
        let element_mode = ti.element_mode;
        let element_display = ti.element_display;
        let insert_index = ti.insert_index;
        let insert_size = Tabs::element_input_size(ti);
        let display_size = element_display_size(element_display);
        let fi = self.files.current(index);
        let ib = &mut ti.insert_vector;
        if c != '.' {
            ib[insert_index] = c as u8;
        }
        let base = element_mode_base(element_mode);
        let patch = &mut fi.patch;
        let undo = &mut fi.undo;
        let block = &mut fi.block;
        let got_input = Self::do_flush_input(*ib, insert_size, display_size, base, &mut vv);
        if got_input {
            let min = pos;
            let max = min + vv.len();
            let key = block.offset + pos as u64;
            undo.push(Data::new(key, (&block.buffer[min..max]).to_vec()));
            block.buffer.splice(min..max, vv.clone());
            undo.push(Data::new(key, (&block.buffer[min..max]).to_vec()));
            Self::do_update_patch(patch, key, vv);
        }
        self.tabs.insert_index_next();
    }

    fn do_undo(&mut self) {
        let fi = self.files.current(self.tabs.file_index());
        for _i in 0..2 {
            let opt = fi.undo.pop();
            if opt.is_some() {
                let data = opt.unwrap();
                Self::do_update_patch(&mut fi.patch, data.offset, data.data.clone());
                fi.redo.push(data);
            }
        }
    }

    fn do_redo(&mut self) {
        let fi = self.files.current(self.tabs.file_index());
        for _i in 0..2 {
            let opt = fi.redo.pop();
            if opt.is_some() {
                let data = opt.unwrap();
                Self::do_update_patch(&mut fi.patch, data.offset, data.data.clone());
                fi.undo.push(data);
            }
        }
    }

    fn on_key(&mut self, print: &mut Print, c: char, modifier: KeyModifiers) {
        if self.is_insert_mode() {
            if c.is_ascii_hexdigit() || c == '.' {
                self.handle_insert(c);
            } else if c == 'u' {
                self.do_undo();
            } else if c == 'U' {
                self.do_redo();
            }
        } else {
            match c {
                'Q' => {
                    self.should_quit = true;
                }
                ':' => {
                    self.enter_prompt = true;
                }
                'W' => {
                    if !self.tabs.tabs.is_empty() {
                        let r = self.files.write(self.tabs.file_index());
                        if r.is_err() {
                            print
                                .history
                                .print(self.theme.error, r.unwrap_err().to_string());
                        }
                    }
                }
                '[' => {
                    self.decrease_print_width();
                }
                ']' => {
                    self.increase_print_width();
                }
                'p' => {
                    self.next_display();
                }
                'P' => {
                    self.prev_display();
                }
                'o' => {
                    self.next_element();
                }
                'O' => {
                    self.prev_element();
                }
                'i' => {
                    self.next_mode();
                }
                'I' => {
                    self.prev_mode();
                }
                'n' => {
                    self.next_hit(modifier);
                }
                'N' => {
                    self.prev_hit(modifier);
                }
                _ => {}
            }
        }
    }

    fn parse_u64_number(input: &str) -> Result<u64, ParseIntError> {
        let z;
        if input.starts_with("0x") {
            z = u64::from_str_radix(input.strip_prefix("0x").unwrap(), 16);
        } else if input.ends_with("o") {
            z = u64::from_str_radix(input.strip_suffix("o").unwrap(), 8);
        } else if input.ends_with("b") {
            z = u64::from_str_radix(input.strip_suffix("b").unwrap(), 2);
        } else {
            z = input.parse::<u64>();
        }
        z
    }

    pub fn pin_tab(&mut self) {
        if !self.tabs.tabs.is_empty() {
            self.tabs.tabs[self.tabs.index].fileitem_index = self.files.index;
        }
    }

    pub fn on_command(&mut self, print: &mut Print<'a>) {
        let inputs: Vec<&str> = (self.textarea.lines()[0]).split_whitespace().collect();
        if inputs.len() > 1 {
            if inputs[0].eq("file") {
                if inputs[1].eq("next") {
                    self.files.next();
                    self.pin_tab();
                } else if inputs[1].eq("prev") {
                    self.files.previous();
                    self.pin_tab();
                } else if inputs[1].eq("add") && inputs.len() > 2 {
                    self.files.add(inputs[2].to_string(), &mut self.tabs);
                }
            } else if inputs[0].eq("tab") {
                if inputs[1].eq("next") {
                    self.tabs.next();
                } else if inputs[1].eq("prev") {
                    self.tabs.previous();
                }
            } else if inputs[0].eq("search") {
                let ret = self.handle_search(inputs[1].to_string());
                if ret.is_err() {
                    print
                        .history
                        .print(self.theme.error, "Search failed!".to_string());
                } else {
                    print.history.print(
                        self.theme.text,
                        format!("Found {} results", ret.unwrap()).to_string(),
                    );
                }
            } else if inputs[0].eq("block_size") {
                self.set_block_size(Self::parse_u64_number(inputs[1]));
            } else if inputs[0].eq("offset") {
                self.set_block_offset(Self::parse_u64_number(inputs[1]));
            } else if inputs[0].eq("print") {
                if inputs.len() > 2 {
                    self.handle_print(print, inputs[1].to_string(), inputs[2].to_string());
                }
            } else if inputs[0].eq("show") {
                if inputs.len() > 2 {
                    self.handle_show(inputs[1].to_string(), inputs[2].to_string());
                }
            }
        }
    }

    pub fn on_draw(&mut self) -> &Vec<Spans<'a>> {
        if self.tabs.current().display == Display::Asm {
            self.get_asm();
        } else if self.tabs.current().display == Display::Print {
            if self.tabs.current().print_display == PrintDisplay::ASCIIPrint {
                self.get_ascii_print();
            } else if self.tabs.current().print_display == PrintDisplay::ASCIIEscape {
                self.get_ascii_escape();
            } else if self.tabs.current().print_display == PrintDisplay::UnicodePrint {
                self.get_unicode_print();
            } else if self.tabs.current().print_display == PrintDisplay::UnicodeEscape {
                self.get_unicode_escape();
            }
        } else if self.tabs.current().display == Display::Element {
            if self.tabs.current().element_display == ElementDisplay::Byte {
                if self.tabs.current().element_mode == ElementMode::Hex {
                    self.get_hexbyte();
                } else if self.tabs.current().element_mode == ElementMode::Dec {
                    self.get_decbyte();
                } else if self.tabs.current().element_mode == ElementMode::Oct {
                    self.get_octbyte();
                } else if self.tabs.current().element_mode == ElementMode::Bin {
                    self.get_binbyte();
                }
            } else if self.tabs.current().element_display == ElementDisplay::Word {
                if self.tabs.current().element_mode == ElementMode::Hex {
                    self.get_hexword();
                } else if self.tabs.current().element_mode == ElementMode::Dec {
                    self.get_decword();
                } else if self.tabs.current().element_mode == ElementMode::Oct {
                    self.get_octword();
                } else if self.tabs.current().element_mode == ElementMode::Bin {
                    self.get_binword();
                }
            } else if self.tabs.current().element_display == ElementDisplay::DWord {
                if self.tabs.current().element_mode == ElementMode::Hex {
                    self.get_hexdword();
                } else if self.tabs.current().element_mode == ElementMode::Dec {
                    self.get_decdword();
                } else if self.tabs.current().element_mode == ElementMode::Oct {
                    self.get_octdword();
                } else if self.tabs.current().element_mode == ElementMode::Bin {
                    self.get_bindword();
                }
            } else if self.tabs.current().element_display == ElementDisplay::QWord {
                if self.tabs.current().element_mode == ElementMode::Hex {
                    self.get_hexqword();
                } else if self.tabs.current().element_mode == ElementMode::Dec {
                    self.get_decqword();
                } else if self.tabs.current().element_mode == ElementMode::Oct {
                    self.get_octqword();
                } else if self.tabs.current().element_mode == ElementMode::Bin {
                    self.get_binqword();
                }
            }
        } else if self.tabs.current().display == Display::Visual {
            if self.tabs.current().visual_display == VisualDisplay::Color {
                self.get_color();
            } else if self.tabs.current().visual_display == VisualDisplay::Entropy {
                self.get_entropy();
            }
        }
        &self.cache.buffer
    }

    pub fn sync_file(&mut self, print: &mut Print) {
        if Self::need_block(self) {
            let ret = Self::read_block(self);
            if ret.is_err() {
                print
                    .history
                    .print(self.theme.error, "Failed to read block!".to_string());
            }
        }
        if !self.files.files.is_empty() {
            let fi = self.files.current(self.tabs.file_index());
            Files::do_apply_patch(&mut fi.block, &fi.patch);
        }
    }

    pub fn handle_input(&mut self, print: &mut Print<'a>, key: KeyEvent) {
        if self.enter_prompt {
            if key.code == KeyCode::Enter {
                self.on_command(print);
                self.enter_prompt = false;
            } else {
                self.textarea.input(key);
            }
        } else {
            match key.code {
                KeyCode::Char(c) => self.on_key(print, c, key.modifiers),
                KeyCode::Left => self.on_left(print),
                KeyCode::Up => self.on_up(print),
                KeyCode::Right => self.on_right(print),
                KeyCode::Down => self.on_down(print),
                KeyCode::PageUp => self.on_pageup(print),
                KeyCode::PageDown => self.on_pagedown(print),
                KeyCode::Tab => self.on_tab(print),
                KeyCode::End => self.on_end(print),
                KeyCode::Home => self.on_home(print),
                KeyCode::Insert => self.on_insert(print),
                KeyCode::F(1) => self.on_f1(print),
                _ => {}
            }
        }
    }

    pub fn on_tick(&mut self) {
        let now = Instant::now();
        self.progress = now.duration_since(self.now).as_secs_f64();
        self.now = now;
    }

    fn pop(input: &[u8]) -> [u8; 64] {
        let mut array = [0u8; 64];
        for (&x, p) in input.iter().zip(array.iter_mut()) {
            *p = x;
        }
        array
    }

    pub fn get_help(&mut self) -> Vec<Spans<'a>> {
        let text;
        if self.is_insert_mode() {
            text = vec![
                Spans::from("Help"),
                Spans::from("tab       toggle history log"),
                Spans::from("up        move cursor up"),
                Spans::from("down      move cursor down"),
                Spans::from("left      move cursor left"),
                Spans::from("right     move cursor right"),
                Spans::from("<0-fF>    edit nibbles"),
                Spans::from("'.'       skip nibble"),
                Spans::from("u         undo"),
                Spans::from("U         redo"),
                Spans::from("home      jump cursor to start of page"),
                Spans::from("end       jump cursor to end of page"),
                Spans::from("insert    exit insert mode"),
            ];
        } else {
            text = vec![
                Spans::from("Help"),
                Spans::from("':'       enter command line"),
                Spans::from("Q         exit"),
                Spans::from("W         save changes to selected file"),
                Spans::from("[         decrease print width"),
                Spans::from("]         increase print width"),
                Spans::from("p         next display mode"),
                Spans::from("P         prev display mode"),
                Spans::from("o         next element display mode"),
                Spans::from("O         prev element display mode"),
                Spans::from("i         next interpretation mode"),
                Spans::from("I         prev interpretation mode"),
                Spans::from("n         jump to next search hit"),
                Spans::from("N         jump to prev search hit"),
                Spans::from("Ctrl+n    pick next group of search hits"),
                Spans::from("Ctrl+N    pick prev group of search hits"),
                Spans::from("tab       toggle history log"),
                Spans::from("up        scroll up"),
                Spans::from("down      scroll down"),
                Spans::from("pageup    scroll page up"),
                Spans::from("pagedown  scroll page down"),
                Spans::from("home      jump to start of file"),
                Spans::from("end       jump to end of file"),
                Spans::from("insert    enter insert mode"),
            ];
        }
        text
    }
}
