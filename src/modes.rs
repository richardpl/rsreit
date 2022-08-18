#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Display {
    Element,
    Asm,
    Print,
    Visual,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ElementDisplay {
    Byte,
    Word,
    DWord,
    QWord,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PrintDisplay {
    ASCIIPrint,
    ASCIIEscape,
    UnicodePrint,
    UnicodeEscape,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum VisualDisplay {
    Color,
    Entropy,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ElementMode {
    Hex,
    Dec,
    Oct,
    Bin,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AsmDisplay {
    Nasm,
    Masm,
    Gas,
    Intel,
}

pub fn element_display_size(display: ElementDisplay) -> u16 {
    match display {
        ElementDisplay::Byte => 1,
        ElementDisplay::Word => 2,
        ElementDisplay::DWord => 4,
        ElementDisplay::QWord => 8,
    }
}

pub fn element_mode_size(mode: ElementMode) -> u16 {
    match mode {
        ElementMode::Hex => 2,
        ElementMode::Dec => 3,
        ElementMode::Oct => 3,
        ElementMode::Bin => 8,
    }
}

pub fn element_mode_base(mode: ElementMode) -> u32 {
    match mode {
        ElementMode::Hex => 16,
        ElementMode::Dec => 10,
        ElementMode::Oct => 8,
        ElementMode::Bin => 2,
    }
}
