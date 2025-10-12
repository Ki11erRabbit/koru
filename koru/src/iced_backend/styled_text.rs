use iced::widget::text::Rich;
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
                    ..
                } => {
                    todo!("implement styling by looking up the color name and applying it")
                }
            }
        }
    }
    Rich::with_spans(spans)
}