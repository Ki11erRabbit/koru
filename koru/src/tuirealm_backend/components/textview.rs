use log::error;
use tabled::grid::records::vec_records::Cell;
use tuirealm::{AttrValue, Attribute, Component, Event, Frame, MockComponent, Props, State};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::ratatui::layout::Rect;
use koru_core::styled_text::{ColorValue, StyledFile, StyledText};
use tuirealm::props::{Color, TextSpan};
use tuirealm::ratatui::prelude::Text;
use tuirealm::ratatui::text::{Line, Span};
use tuirealm::ratatui::widgets::{Clear, Paragraph};
use crate::tuirealm_backend::colors::ColorDefinitions;
use crate::tuirealm_backend::UiMessage;

pub struct TextView {
    props: Props,
}

impl TextView {
    pub fn new() -> Self {
        TextView {
            props: Props::default(),
        }
    }

    pub fn lines(text: &StyledFile, top_line: usize, line_count: usize) -> AttrValue{
        let mut lines = Vec::new();
        
        for line in text.lines().into_iter().skip(top_line).take(line_count) {
            let mut new_line = Vec::new();
            for item in line {
                match item {
                    StyledText::None { text} => {
                        new_line.push(TextSpan::new(text.to_string()));
                    }
                    StyledText::Style { text, fg_color, bg_color, .. } => {
                        
                        let fg_color = match ColorDefinitions::get(fg_color) {
                            ColorValue::Rgb { r, g , b } => {
                                Color::Rgb(r, g, b)
                            }
                            ColorValue::Ansi(ansi) => {
                                Color::Indexed(ansi)
                            }
                        };

                        let bg_color = match ColorDefinitions::get(bg_color) {
                            ColorValue::Rgb { r, g , b } => {
                                Color::Rgb(r, g, b)
                            }
                            ColorValue::Ansi(ansi) => {
                                Color::Indexed(ansi)
                            }
                        };
                        
                        new_line.push(
                            TextSpan::new(text.to_string()).bg(bg_color).fg(fg_color)
                        );
                    }
                    
                }
            }
            lines.push(new_line);
        }

        AttrValue::Table(lines)
    }
}

impl MockComponent for TextView {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let Some(AttrValue::Table(text)) = self.query(Attribute::Text) else {
            return;
        };
        let Some(AttrValue::Number(column_offset)) = self.query(Attribute::Custom("ColumnOffset")) else {
            return;
        };
        let Some(AttrValue::Color(bg_color)) = self.query(Attribute::Custom("Background")) else {
            return;
        };
        let mut lines = Vec::new();

        for line in text {
            let mut total_width = 0;
            let mut line = line.into_iter().map(|text_span| {
                total_width += text_span.content.width();
                let style = tuirealm::ratatui::style::Style::default()
                    .fg(text_span.fg)
                    .bg(text_span.bg)
                    .add_modifier(text_span.modifiers);
                Span::styled(text_span.content, style)
            }).collect::<Vec<Span>>();
            let visible_width = total_width.saturating_sub(column_offset as usize);
            if visible_width < area.width as usize {
                let pad = " ".repeat(area.width as usize - visible_width);
                line.push(Span::styled(pad, tuirealm::ratatui::style::Style::default().bg(bg_color)));
            }
            lines.push(Line::from(
                line
            ));
        }
        while lines.len() < area.height as usize {
            let pad = " ".repeat(area.width as usize);
            lines.push(Line::from(vec![Span::styled(pad, tuirealm::ratatui::style::Style::default().bg(bg_color))]))
        }

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .style(tuirealm::ratatui::style::Style::default().bg(bg_color));

        frame.render_widget(paragraph, area)
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<UiMessage, UiMessage> for TextView {
    fn on(&mut self, ev: Event<UiMessage>) -> Option<UiMessage> {
        match ev {
            _ => None
        }
    }
}