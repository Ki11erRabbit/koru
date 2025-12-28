use iced_core::Color;
use iced_core::text::Span;
use koru_core::styled_text::{ColorType, StyledText};
use scrollable_rich::rich::{Rich, VisibleTextMetrics};
use crate::iced_backend::UiMessage;

pub fn rich<'a, Theme, Renderer>(text: &'a Vec<Vec<StyledText>>, line_offset: usize, line_count_callback: impl Fn(VisibleTextMetrics) + 'a) -> Rich<'a, UiMessage, Theme, Renderer> 
where 
    Theme: iced::widget::text::Catalog,
    Renderer: iced::advanced::text::Renderer + 'a,
{
    let mut spans = Vec::new();
    let mut line_starts = Vec::new();
    let mut span_index = 0;
    for line in text {
        line_starts.push(span_index);
        for item in line {
            span_index += 1;
            match item {
                StyledText::None{ text} => {
                    let span = Span::new(text.to_string());
                    spans.push(span)
                }
                StyledText::Style {
                    text,
                    bg_color,
                    ..
                } => {
                    match bg_color {
                        ColorType::Cursor => {
                            let span = Span::new(text.to_string())
                                .background(Color::from_rgb8(100, 100, 100))
                                .color(Color::from_rgb8(255, 255, 255));
                            spans.push(span);
                        }
                        ColorType::Selection => {
                            let span = Span::new(text.to_string())
                                .background(Color::from_rgb8(170, 170, 170))
                                .color(Color::from_rgb8(255, 255, 255));
                            spans.push(span);
                        }
                        _ => spans.push(Span::new(text.to_string())),
                    }
                    
                }
            }
        }
    }
    Rich::with_spans(spans, line_starts.into_boxed_slice(), line_offset, line_count_callback)
}