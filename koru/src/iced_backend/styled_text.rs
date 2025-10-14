use iced::widget::text::Rich;
use iced_core::Color;
use iced_core::text::Span;
use koru_core::styled_text::StyledText;
use crate::iced_backend::UiMessage;

pub fn rich<'a, Theme, Renderer>(text: &'a Vec<Vec<StyledText>>) -> Rich<'a, UiMessage, Theme, Renderer> 
where 
    Theme: iced::widget::text::Catalog,
    Renderer: iced::advanced::text::Renderer + 'a,
{
    let mut spans = Vec::new();
    for line in text {
        for item in line {
            match item {
                StyledText::None(text) => {
                    let span = Span::new(text);
                    spans.push(span)
                }
                StyledText::Style {
                    text,
                    ..
                } => {
                    let span = Span::new(text)
                        .background(Color::from_rgb8(100, 100, 100))
                        .color(Color::from_rgb8(255, 255, 255));
                    spans.push(span);
                }
            }
        }
    }
    Rich::with_spans(spans)
}