use iced_core::{alignment, event, layout, mouse, renderer, Clipboard, Color, Element, Event, Layout, Length, Pixels, Point, Rectangle, Shell, Size, Vector, Widget};
use iced_core::layout::{Limits, Node};
use iced_core::text::{Fragment, LineHeight, Paragraph, Shaping, Span, Wrapping};
use iced_core::widget::text::{Catalog, Style, StyleFn};
use iced_core::widget::{text, tree, Tree};
use iced_core::widget::tree::Tag;

#[derive(Debug, Clone, Copy, Default)]
pub struct VisibleTextMetrics {
    pub line_count: usize,
    /// None if not monospaced or can't be determined
    pub max_columns: Option<usize>,
}

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
    column_offset: usize,
    metrics_callback: Box<dyn Fn(VisibleTextMetrics) + 'a>,
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
    pub fn visible_line_count(&self, viewport_height: f32, renderer: &Renderer) -> usize {
        let line_height_px = self.calculate_line_height(renderer);
        let line_height_fract = line_height_px.fract();
        // We need to correct the line height to be either the fractional part or 0.5 if it is zero
        // This ensures that we don't think we draw an extra line than we think we have
        let line_height_fract = if line_height_fract == 0.0 {
            0.5
        } else {
            line_height_fract
        };

        // The number of available lines from the current offset
        let available_lines = if self.line_offset < self.line_starts.len() {
            self.line_starts.len() - self.line_offset
        } else {
            0
        };

        if line_height_px <= 0.0 || viewport_height <= 0.0 {
            return 0;
        }

        // Calculate how many lines fit
        let exact = viewport_height / line_height_px;
        let calculated = exact.floor() as usize;

        // Check if there's a fractional remainder (incomplete line space)
        let has_remainder = (exact - exact.floor()) <= line_height_fract;

        // If there's extra space that doesn't fit a complete line, subtract 1
        let lines_that_fit = if has_remainder {
            calculated.saturating_sub(1)
        } else {
            calculated
        };

        // Return the minimum of what fits and what's available
        lines_that_fit.min(available_lines)
    }

    /// Get the range of lines that will actually be rendered
    pub fn visible_line_range(&self, viewport_height: f32, renderer: &Renderer) -> std::ops::Range<usize> {
        if self.line_offset >= self.line_starts.len() {
            return 0..0; // Empty range if offset is beyond content
        }

        let line_height_px = self.calculate_line_height(renderer);
        let line_height_fract = line_height_px.fract();
        // We need to correct the line height to be either the fractional part or 0.5 if it is zero
        // This ensures that we don't think we draw an extra line than we think we have
        let line_height_fract = if line_height_fract == 0.0 {
            0.5
        } else {
            line_height_fract
        };

        if line_height_px <= 0.0 || viewport_height <= 0.0 {
            return self.line_offset..self.line_offset;
        }

        // The number of available lines from the current offset
        let available_lines = self.line_starts.len() - self.line_offset;

        // Calculate how many lines fit
        let exact = viewport_height / line_height_px;
        let calculated = exact.floor() as usize;

        // Check if there's a fractional remainder (incomplete line space)
        let has_remainder = (exact - exact.floor()) <= line_height_fract;

        // If there's extra space that doesn't fit a complete line, subtract 1
        let lines_that_fit = if has_remainder {
            calculated.saturating_sub(1)
        } else {
            calculated
        };

        let lines_that_fit = lines_that_fit.min(available_lines);

        let start = self.line_offset;
        let end = start + lines_that_fit;

        start..end
    }

    fn calculate_line_height(&self, renderer: &Renderer) -> f32 {
        match self.line_height {
            LineHeight::Absolute(px) => px.0,
            LineHeight::Relative(factor) => {
                let font_size = self.size.unwrap_or(renderer.default_size()).0;
                font_size * factor
            }
        }
    }

    /// Calculate the maximum number of columns that can fit in the viewport width
    /// Returns None if the font is not monospaced or character width cannot be determined
    fn calculate_max_columns(&self, viewport_width: f32, renderer: &Renderer) -> Option<usize> {
        // Get the font size
        let font_size = self.size.unwrap_or(renderer.default_size());
        let default_font = renderer.default_font();
        let font = self.font.as_ref().unwrap_or(&default_font);

        // Measure using a string of characters to get more accurate average width
        // This accounts for kerning and character spacing better than single character
        let char_width = self.measure_char_width(font_size, font, renderer)?;

        if char_width > 0.0 {
            // Use floor to be conservative - we want to ensure characters fit
            // Add a small epsilon to account for floating point errors
            let columns = (viewport_width / char_width).floor() as usize;
            // Return at least 1 column if viewport has any width
            if columns > 0 || viewport_width > 0.0 {
                Some(columns.max(1))
            } else {
                Some(0)
            }
        } else {
            None
        }
    }

    /// Measure the average width of characters for this font
    /// Returns None if measurement fails or font is not monospaced
    fn measure_char_width(
        &self,
        size: Pixels,
        font: &Renderer::Font,
        renderer: &Renderer,
    ) -> Option<f32> {
        // Test with multiple characters to verify monospace and get accurate width
        // Use a representative string that includes common characters
        let test_chars = "MWiI0O";

        let test_text = iced_core::Text {
            content: test_chars,
            bounds: Size::new(f32::INFINITY, f32::INFINITY),
            size,
            line_height: self.line_height,
            font: font.clone(),
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            shaping: Shaping::Basic, // Use Basic shaping for more predictable monospace behavior
            wrapping: Wrapping::None,
        };

        let paragraph = Renderer::Paragraph::with_text(test_text);
        let total_bounds = paragraph.min_bounds();

        if total_bounds.width <= 0.0 {
            return None;
        }

        // Calculate average character width
        let char_count = test_chars.chars().count() as f32;
        let avg_width = total_bounds.width / char_count;

        // Verify this is actually monospaced by testing individual characters
        // For a monospaced font, each character should have roughly the same width
        let tolerance = 0.1; // 10% tolerance for rounding errors

        for test_char in ['M', 'i', '0'].iter() {
            let single_char = test_char.to_string();
            let single_text = iced_core::Text {
                content: single_char.as_str(),
                bounds: Size::new(f32::INFINITY, f32::INFINITY),
                size,
                line_height: self.line_height,
                font: font.clone(),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                shaping: Shaping::Basic,
                wrapping: Wrapping::None,
            };

            let single_paragraph = Renderer::Paragraph::with_text(single_text);
            let single_bounds = single_paragraph.min_bounds();

            if single_bounds.width > 0.0 {
                let ratio = (single_bounds.width - avg_width).abs() / avg_width;
                if ratio > tolerance {
                    // Not monospaced enough
                    return None;
                }
            }
        }

        Some(avg_width)
    }

    /// Calculate visible text metrics including line count and column count
    fn calculate_metrics(&self, viewport: &Rectangle, renderer: &Renderer) -> VisibleTextMetrics {
        let line_count = self.visible_line_count(viewport.height, renderer);
        let max_columns = self.calculate_max_columns(viewport.width, renderer);

        VisibleTextMetrics {
            line_count,
            max_columns,
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
            column_offset: 0,
            metrics_callback: Box::new(|_| {}),
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
        column_offset: usize,
        metrics_callback: impl Fn(VisibleTextMetrics) + 'a,
    ) -> Self {
        Self {
            spans: Box::new(spans),
            line_starts,
            line_offset,
            column_offset,
            metrics_callback: Box::new(metrics_callback),
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

    /// Sets the default [`LineHeight`] of the [`Rich`] text.
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
    column_offset: usize,
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
            self.column_offset,
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

        // FIXED: Use state.spans which are the actual spans in the paragraph
        // (these have been processed with line_offset and column_offset applied)
        for (index, span) in state.spans.iter().enumerate() {
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

        // Calculate and report metrics
        let metrics = self.calculate_metrics(viewport, renderer);
        (self.metrics_callback)(metrics);

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
            column_offset: 0,
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
                                // FIXED: Use state.spans which correspond to the paragraph
                                if let Some(link) = state
                                    .spans
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

            // FIXED: Use state.spans which correspond to the paragraph
            if let Some(span) = state
                .paragraph
                .hit_span(position)
                .and_then(|span| state.spans.get(span))
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
    column_offset: usize,
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

        // Apply column offset if wrapping is disabled
        let processed_spans: Vec<Span<'_, Link, Renderer::Font>>;
        let final_spans = if wrapping == Wrapping::None && column_offset > 0 {
            processed_spans = apply_column_offset_to_spans(spans, column_offset);
            &processed_spans[..]
        } else {
            spans
        };

        let text_with_spans = || iced_core::Text {
            content: final_spans,
            bounds,
            size,
            line_height,
            font,
            horizontal_alignment,
            vertical_alignment,
            shaping: Shaping::Advanced,
            wrapping,
        };

        // Check if we need to update the paragraph
        let needs_update = state.spans != final_spans
            || state.line_offset != line_offset
            || state.column_offset != column_offset;

        if needs_update {
            state.paragraph =
                Renderer::Paragraph::with_spans(text_with_spans());
            state.spans = final_spans.iter().cloned().map(Span::to_static).collect();
            state.line_offset = line_offset;
            state.column_offset = column_offset;
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

/// Helper function to apply column offset to a slice of spans
fn apply_column_offset_to_spans<'a, Link, Font>(
    spans: &[Span<'a, Link, Font>],
    column_offset: usize,
) -> Vec<Span<'a, Link, Font>>
where
    Link: Clone,
    Font: Clone,
{
    if column_offset == 0 {
        return spans.to_vec();
    }

    let mut result = Vec::new();
    let mut chars_processed = 0;

    for span in spans {
        let text = &span.text;

        // Handle newlines specially - they reset column counting per line
        if text.contains('\n') {
            // Split by newline and process each segment
            let lines: Vec<&str> = text.split('\n').collect();

            for (i, line) in lines.iter().enumerate() {
                let line_char_count = line.chars().count();

                if chars_processed < column_offset {
                    let chars_to_skip = column_offset.saturating_sub(chars_processed);

                    if chars_to_skip < line_char_count {
                        // Take part of this line
                        let trimmed: String = line.chars().skip(chars_to_skip).collect();
                        if !trimmed.is_empty() {
                            let mut new_span = span.clone();
                            let final_text = if i < lines.len() - 1 {
                                format!("{}\n", trimmed)
                            } else {
                                trimmed
                            };
                            new_span.text = Fragment::from(final_text);
                            result.push(new_span);
                        } else if i < lines.len() - 1 {
                            // Just add newline
                            let mut new_span = span.clone();
                            new_span.text = Fragment::from("\n");
                            result.push(new_span);
                        }
                        chars_processed = column_offset; // We've now applied the offset
                    } else if i < lines.len() - 1 {
                        // Skip this line but keep the newline
                        let mut new_span = span.clone();
                        new_span.text = Fragment::from("\n");
                        result.push(new_span);
                        chars_processed = 0; // Reset for next line
                    }
                } else {
                    // We've already applied offset, include full line
                    let mut new_span = span.clone();
                    let final_text = if i < lines.len() - 1 {
                        format!("{}\n", line)
                    } else {
                        line.to_string()
                    };
                    new_span.text = Fragment::from(final_text);
                    result.push(new_span);
                }

                if i < lines.len() - 1 {
                    chars_processed = 0; // Reset column count after newline
                }
            }
        } else {
            // No newlines, simpler logic
            let char_count = text.chars().count();

            if chars_processed + char_count <= column_offset {
                // Skip this entire span
                chars_processed += char_count;
                continue;
            }

            if chars_processed < column_offset {
                // Partially skip this span
                let chars_to_skip = column_offset - chars_processed;
                let trimmed: String = text.chars().skip(chars_to_skip).collect();

                if !trimmed.is_empty() {
                    let mut new_span = span.clone();
                    new_span.text = Fragment::from(trimmed);
                    result.push(new_span);
                }
                chars_processed += char_count;
            } else {
                // Include this span entirely
                result.push(span.clone());
                chars_processed += char_count;
            }
        }
    }

    result
}

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