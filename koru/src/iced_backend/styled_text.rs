use iced_core::Color;
use iced_core::text::{Span, Wrapping};
use koru_core::styled_text::{ColorType, StyledText};
use scrollable_rich::rich::{Rich, VisibleTextMetrics};
use crate::iced_backend::colors::ColorDefinitions;
use crate::iced_backend::UiMessage;

pub fn rich<'a, Theme, Renderer>(
    text: &'a Vec<Vec<StyledText>>,
    line_offset: usize,
    column_offset: usize,
    line_count_callback: impl Fn(VisibleTextMetrics) + 'a
) -> Rich<'a, UiMessage, Theme, Renderer>
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
                    fg_color,
                    bg_color,
                    ..
                } => {
                    let fg_color = ColorDefinitions::get(fg_color);
                    let bg_color = ColorDefinitions::get(bg_color);
                    let span = Span::new(text.to_string())
                        .background(Color::from_rgb8(bg_color.0, bg_color.1, bg_color.2))
                        .color(Color::from_rgb8(fg_color.0, fg_color.1, fg_color.2));
                    spans.push(span);
                }
            }
        }
    }
    Rich::with_spans(spans, line_starts.into_boxed_slice(), line_offset, column_offset, line_count_callback)
}

pub fn rich_simple<'a, Theme, Renderer>(
    text: Vec<String>,
) -> iced::widget::text::Rich<'a, UiMessage, Theme, Renderer>
where
    Theme: iced::widget::text::Catalog,
    Renderer: iced::advanced::text::Renderer + 'a,
{
    let mut spans = Vec::new();
    for line in text {
        spans.push(Span::new(line));
    }
    iced::widget::text::Rich::with_spans(spans)
}