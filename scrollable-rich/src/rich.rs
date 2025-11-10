use iced_core::{alignment, event, layout, mouse, renderer, Clipboard, Color, Element, Event, Layout, Length, Pixels, Point, Rectangle, Shell, Size, Vector, Widget};
use iced_core::layout::{Limits, Node};
use iced_core::text::{LineHeight, Paragraph, Shaping, Span, Wrapping};
use iced_core::widget::text::{Catalog, Style, StyleFn};
use iced_core::widget::{text, tree, Tree};
use iced_core::widget::tree::Tag;

#[allow(missing_debug_implementations)]
pub struct Rich<'a, Link, Theme = iced_core::Theme, Renderer = iced_renderer::Renderer>
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: iced_core::text::Renderer,
{
    spans: Box<dyn AsRef<[Span<'a, Link, Renderer::Font>]> + 'a>,
    line_starts: Box<[usize]>,
    line_offset: usize,
    line_height_callback: Box<dyn Fn(usize) + 'a>,
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

impl<'a, Link, Theme, Renderer> Rich<'_, Link, Theme, Renderer>
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: iced_core::text::Renderer,
    Renderer::Font: 'a,
{
    pub fn visible_line_count(&self, viewport_height: f32) -> usize {
        let line_height_px = self.calculate_line_height();
        (viewport_height / line_height_px).ceil() as usize
    }

    pub fn visible_line_range(&self, viewport_height: f32) -> std::ops::Range<usize> {
        let visible_count = self.visible_line_count(viewport_height);
        let start = self.line_offset;
        let end = (start + visible_count).min(self.line_starts.len());
        start..end
    }

    fn calculate_line_height(&self) -> f32 {
        match self.line_height {
            LineHeight::Absolute(px) => px.0,
            LineHeight::Relative(factor) => {
                let font_size = self.size.unwrap_or(Pixels(14.0)).0;
                font_size * factor
            }
        }
    }
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
            line_starts: Box::new([]),
            line_offset: 0,
            line_height_callback: Box::new(|_| {}),
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

    pub fn with_spans(
        spans: impl AsRef<[Span<'a, Link, Renderer::Font>]> + 'a,
        line_starts: Box<[usize]>,
        line_offset: usize,
        line_height_callback: impl Fn(usize) + 'a,
    ) -> Self {
        Self {
            spans: Box::new(spans),
            line_starts,
            line_offset,
            line_height_callback: Box::new(line_height_callback),
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

    /// Sets the default size of the [`Rich`] text.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the defualt [`LineHeight`] of the [`Rich`] text.
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the default font of the [`Rich`] text.
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the width of the [`Rich`] text boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Rich`] text boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Centers the [`Rich`] text, both horizontally and vertically.
    pub fn center(self) -> Self {
        self.align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
    }

    /// Sets the [`alignment::Horizontal`] of the [`Rich`] text.
    pub fn align_x(
        mut self,
        alignment: impl Into<alignment::Horizontal>,
    ) -> Self {
        self.align_x = alignment.into();
        self
    }

    /// Sets the [`alignment::Vertical`] of the [`Rich`] text.
    pub fn align_y(
        mut self,
        alignment: impl Into<alignment::Vertical>,
    ) -> Self {
        self.align_y = alignment.into();
        self
    }

    /// Sets the [`Wrapping`] strategy of the [`Rich`] text.
    pub fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.wrapping = wrapping;
        self
    }

    /// Sets the default style of the [`Rich`] text.
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the default [`Color`] of the [`Rich`] text.
    pub fn color(self, color: impl Into<Color>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.color_maybe(Some(color))
    }

    /// Sets the default [`Color`] of the [`Rich`] text, if `Some`.
    pub fn color_maybe(self, color: Option<impl Into<Color>>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        let color = color.map(Into::into);

        self.style(move |_theme| Style { color })
    }

    /// Sets the default style class of the [`Rich`] text.
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Link, Theme, Renderer> Default for Rich<'a, Link, Theme, Renderer>
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: iced_core::text::Renderer,
    Renderer::Font: 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

struct State<Link, P: Paragraph> {
    spans: Vec<Span<'static, Link, P::Font>>,
    line_starts: Vec<usize>,
    line_offset: usize,
    span_pressed: Option<usize>,
    paragraph: P,
}

impl<'a, Link, Theme, Renderer> Widget<Link, Theme, Renderer>
for Rich<'a, Link, Theme, Renderer>
where
    Link: Clone + 'static,
    Theme: Catalog,
    Renderer: iced_core::text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        layout(
            tree.state
                .downcast_mut::<State<Link, Renderer::Paragraph>>(),
            renderer,
            limits,
            self.width,
            self.height,
            self.spans.as_ref().as_ref(),
            self.line_starts.as_ref(),
            self.line_offset,
            self.line_height,
            self.size,
            self.font,
            self.align_x,
            self.align_y,
            self.wrapping,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree
            .state
            .downcast_ref::<State<Link, Renderer::Paragraph>>();

        let style = theme.style(&self.class);

        let hovered_span = cursor
            .position_in(layout.bounds())
            .and_then(|position| state.paragraph.hit_span(position));

        let index = self.line_starts.get(self.line_offset).unwrap_or(&0);

        for (index, span) in self.spans.as_ref().as_ref()[*index..].iter().enumerate() {
            let is_hovered_link =
                span.link.is_some() && Some(index) == hovered_span;

            if span.highlight.is_some()
                || span.underline
                || span.strikethrough
                || is_hovered_link
            {
                let translation = layout.position() - Point::ORIGIN;
                let regions = state.paragraph.span_bounds(index);

                if let Some(highlight) = span.highlight {
                    for bounds in &regions {
                        let bounds = Rectangle::new(
                            bounds.position()
                                - Vector::new(
                                span.padding.left,
                                span.padding.top,
                            ),
                            bounds.size()
                                + Size::new(
                                span.padding.horizontal(),
                                span.padding.vertical(),
                            ),
                        );

                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: bounds + translation,
                                border: highlight.border,
                                ..Default::default()
                            },
                            highlight.background,
                        );
                    }
                }

                if span.underline || span.strikethrough || is_hovered_link {
                    let size = span
                        .size
                        .or(self.size)
                        .unwrap_or(renderer.default_size());

                    let line_height = span
                        .line_height
                        .unwrap_or(self.line_height)
                        .to_absolute(size);

                    let color = span
                        .color
                        .or(style.color)
                        .unwrap_or(defaults.text_color);

                    let baseline = translation
                        + Vector::new(
                        0.0,
                        size.0 + (line_height.0 - size.0) / 2.0,
                    );

                    if span.underline || is_hovered_link {
                        for bounds in &regions {
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: Rectangle::new(
                                        bounds.position() + baseline
                                            - Vector::new(0.0, size.0 * 0.08),
                                        Size::new(bounds.width, 1.0),
                                    ),
                                    ..Default::default()
                                },
                                color,
                            );
                        }
                    }

                    if span.strikethrough {
                        for bounds in &regions {
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: Rectangle::new(
                                        bounds.position() + baseline
                                            - Vector::new(0.0, size.0 / 2.0),
                                        Size::new(bounds.width, 1.0),
                                    ),
                                    ..Default::default()
                                },
                                color,
                            );
                        }
                    }
                }
            }
        }
        
        let line_height = self.visible_line_range(viewport.height).count();

        (self.line_height_callback)(line_height);

        text::draw(
            renderer,
            defaults,
            layout,
            &state.paragraph,
            style,
            viewport,
        );
    }

    fn tag(&self) -> Tag {
        tree::Tag::of::<State<Link, Renderer::Paragraph>>()
    }

    fn state(&self) -> iced_core::widget::tree::State {
        tree::State::new(State::<Link, _> {
            spans: Vec::new(),
            line_starts: Vec::new(),
            line_offset: 0,
            span_pressed: None,
            paragraph: Renderer::Paragraph::default(),
        })
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Link>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position_in(layout.bounds()) {
                    let state = tree
                        .state
                        .downcast_mut::<State<Link, Renderer::Paragraph>>();

                    if let Some(span) = state.paragraph.hit_span(position) {
                        state.span_pressed = Some(span);

                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let state = tree
                    .state
                    .downcast_mut::<State<Link, Renderer::Paragraph>>();

                if let Some(span_pressed) = state.span_pressed {
                    state.span_pressed = None;

                    if let Some(position) = cursor.position_in(layout.bounds())
                    {
                        match state.paragraph.hit_span(position) {
                            Some(span) if span == span_pressed => {
                                if let Some(link) = self
                                    .spans
                                    .as_ref()
                                    .as_ref()
                                    .get(span)
                                    .and_then(|span| span.link.clone())
                                {
                                    shell.publish(link);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if let Some(position) = cursor.position_in(layout.bounds()) {
            let state = tree
                .state
                .downcast_ref::<State<Link, Renderer::Paragraph>>();

            if let Some(span) = state
                .paragraph
                .hit_span(position)
                .and_then(|span| self.spans.as_ref().as_ref().get(span))
            {
                if span.link.is_some() {
                    return mouse::Interaction::Pointer;
                }
            }
        }

        mouse::Interaction::None
    }
}

fn layout<Link, Renderer>(
    state: &mut State<Link, Renderer::Paragraph>,
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    spans: &[Span<'_, Link, Renderer::Font>],
    line_starts: &[usize],
    line_offset: usize,
    line_height: LineHeight,
    size: Option<Pixels>,
    font: Option<Renderer::Font>,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    wrapping: Wrapping,
) -> layout::Node
where
    Link: Clone,
    Renderer: iced_core::text::Renderer + iced_core::text::Renderer,
{
    layout::sized(limits, width, height, |limits| {
        let bounds = limits.max();

        let size = size.unwrap_or_else(|| renderer.default_size());
        let font = font.unwrap_or_else(|| renderer.default_font());

        let offset = line_starts.get(line_offset).unwrap_or(&0);

        let spans = &spans[*offset..];

        let text_with_spans = || iced_core::Text {
            content: spans,
            bounds,
            size,
            line_height,
            font,
            horizontal_alignment,
            vertical_alignment,
            shaping: Shaping::Advanced,
            wrapping,
        };

        if state.spans != spans {
            state.paragraph =
                Renderer::Paragraph::with_spans(text_with_spans());
            state.spans = spans.iter().cloned().map(Span::to_static).collect();
        } else {
            match state.paragraph.compare(iced_core::Text {
                content: (),
                bounds,
                size,
                line_height,
                font,
                horizontal_alignment,
                vertical_alignment,
                shaping: Shaping::Advanced,
                wrapping,
            }) {
                iced_core::text::Difference::None => {}
                iced_core::text::Difference::Bounds => {
                    state.paragraph.resize(bounds);
                }
                iced_core::text::Difference::Shape => {
                    state.paragraph =
                        Renderer::Paragraph::with_spans(text_with_spans());
                }
            }
        }

        state.paragraph.min_bounds()
    })
}

/*impl<'a, Link, Theme, Renderer> FromIterator<Span<'a, Link, Renderer::Font>>
for Rich<'a, Link, Theme, Renderer>
where
    Link: Clone + 'a,
    Theme: Catalog,
    Renderer: iced_core::text::Renderer,
    Renderer::Font: 'a,
{
    fn from_iter<T: IntoIterator<Item = Span<'a, Link, Renderer::Font>>>(
        spans: T,
    ) -> Self {
        Self::with_spans(spans.into_iter().collect::<Vec<_>>())
    }
}*/

impl<'a, Link, Theme, Renderer> From<Rich<'a, Link, Theme, Renderer>>
for Element<'a, Link, Theme, Renderer>
where
    Link: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: iced_core::text::Renderer + 'a,
{
    fn from(
        text: Rich<'a, Link, Theme, Renderer>,
    ) -> Element<'a, Link, Theme, Renderer> {
        Element::new(text)
    }
}