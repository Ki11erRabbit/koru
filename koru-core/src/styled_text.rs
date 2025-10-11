use bitflags::bitflags;

bitflags! {
    pub struct TextAttribute: u8 {
        const Italic = 0b0000_0001;
        const Bold = 0b0000_0010;
        const Strikethrough = 0b0000_0100;
        const Underline = 0b0000_1000;
    }
}

pub struct Color {
    color_name: String,
}

pub enum StyledText {
    None(String),
    Style {
        color: Color,
        attribute: TextAttribute,
        text: String,
    }
}