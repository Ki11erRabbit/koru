use iced::advanced::{Layout, Widget};
use iced::{Element, Length, Rectangle, Size};
use iced::advanced::graphics::text::cosmic_text::Color;
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::Tree;
use iced::mouse::Cursor;
use iced_core::{alignment, text, Point, Text, Theme};
use iced_core::text::{LineHeight, Wrapping};
use iced_core::widget::text::Catalog;

pub struct EditorViewContent {
    lines: Vec<String>,
}

impl EditorViewContent {
    pub fn new() -> Self {
        EditorViewContent { lines: Vec::new() }
    }

    pub fn longest_line_len(&self) -> usize {
        let mut longest_len = 0;
        for line in &self.lines {
            let current = line.chars().count();
            if current > longest_len {
                longest_len = current;
            }
        }
        longest_len
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

impl From<String> for EditorViewContent {
    fn from(s: String) -> Self {
        let split = s.split('\n');
        EditorViewContent { lines: split.map(String::from).collect() }
    }
}

impl From<&str> for EditorViewContent {
    fn from(s: &str) -> Self {
        let split = s.split('\n');
        EditorViewContent { lines: split.map(String::from).collect() }
    }
}


pub struct EditorView<'a, Theme: Catalog> {
    content: &'a EditorViewContent,
    width: Length,
    height: Length,
    class: Theme::Class<'a>
}

impl<'a, Theme: Catalog> EditorView<'a, Theme> {
    pub fn new(content: &'a EditorViewContent) -> Self {
        Self {
            content,
            width: Length::Fill,
            height: Length::Fill,
            class: Theme::default()
        }
    }
}
impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for EditorView<'a, Theme>
where
    Renderer: iced::advanced::text::Renderer,
    Theme: Catalog
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {


        let limits = limits.width(self.width).height(self.height);

        Node::new(limits.max())
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle
    ) {
        let mut bounds = layout.bounds();

        let font = renderer.default_font();

        //let longest_line_len = self.content.longest_line_len();
        //let height = self.content.line_count();

        for line in &self.content.lines {
            renderer.fill_text(
                Text {
                    content: line.clone(),
                    bounds: bounds.size(),
                    size: renderer.default_size(),
                    line_height: LineHeight::default(),
                    font,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: Wrapping::None,
                },
                bounds.position(),
                iced_core::Color::from_rgb8(0, 0, 0),
                bounds
            );

            let line_height = LineHeight::default();
            match line_height {
                LineHeight::Relative(height) => {
                    bounds = Rectangle::new(Point::new(bounds.x, bounds.y + height), Size::new(bounds.width, bounds.height));
                }
                _ => unreachable!("lin height pixels")
            }
        }
        
    }
}

impl<'a, Message, Theme, Renderer> From<EditorView<'a, Theme>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: text::Renderer,
{
    fn from(widget: EditorView<'a, Theme>) -> Self {
        Self::new(widget)
    }
}