use iced_core::{alignment, Length, Pixels};
use iced_core::text::{LineHeight, Span, Wrapping};
use iced_core::widget::text::Catalog;

#[allow(missing_debug_implementations)]
pub struct Rich<'a, Link, Theme = iced_core::Theme, Renderer = iced_renderer::Renderer>
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: iced_core::text::Renderer,
{
    spans: Box<dyn AsRef<[Span<'a, Link, Renderer::Font>]> + 'a>,
    line_endings: Box<[usize]>,
    size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    font: Option<Renderer::Font>,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    wrapping: Wrapping,
    class: Theme::Class<'a>,
}

impl<'a, Link, Theme, Renderer> Rich<'a, Link, Theme, Renderer> 
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: iced_core::text::Renderer,
    Renderer::Font: 'a,
{
    pub fn new() -> Self {
        Self {
            spans: Box::new([]),
            line_endings: Box::new([]),
            size: None,
            line_height: LineHeight::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            font: None,
            align_x: alignment::Horizontal::Left,
            align_y: alignment::Vertical::Top,
            wrapping: Wrapping::default(),
            class: Theme::default(),
        }
    }
}