use tuirealm::{AttrValue, Attribute, Component, Event, Frame, MockComponent, Props, State, StateValue};
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::event::Key;
use tuirealm::ratatui::layout::Rect;
use koru_core::styled_text::{StyledFile, StyledText};
use tuirealm::props::{Color, TextSpan};
use tuirealm::ratatui::prelude::Text;
use tuirealm::ratatui::text::{Line, Span};
use tuirealm::ratatui::widgets::Paragraph;
use crate::tuirealm_backend::UiMessage;

pub struct TextView {
    props: Props,
    top_line: usize,
}

impl TextView {
    pub fn new() -> Self {
        TextView {
            props: Props::default(),
            top_line: 0,
        }
    }

    pub fn lines(text: &StyledFile) -> AttrValue{
        let mut lines = Vec::new();

        for line in text.lines() {
            let mut new_line = Vec::new();
            for item in line {
                match item {
                    StyledText::None { text} => {
                        new_line.push(TextSpan::new(text.to_string()));
                    }
                    StyledText::Style { text, .. } => {
                        new_line.push(TextSpan::new(text.to_string()).bg(Color::Gray).fg(Color::Black));
                    }
                }
            }
            lines.push(new_line);
        }

        AttrValue::Table(lines)
    }

    pub fn set_starting_line(&mut self, starting_line: usize) {
        self.top_line = starting_line;
    }
}

impl MockComponent for TextView {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let Some(AttrValue::Table(text)) = self.query(Attribute::Text) else {
            return;
        };

        let mut lines = Vec::new();

        for line in text {
            lines.push(Line::from(
                line.into_iter().map(|text_span| {
                    let style = tuirealm::ratatui::style::Style::default()
                        .fg(text_span.fg)
                        .bg(text_span.bg)
                        .add_modifier(text_span.modifiers);
                    Span::styled(text_span.content, style)
                }).collect::<Vec<Span>>(),
            ));
        }

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .scroll((self.top_line as u16, 0));
        
        frame.render_widget(paragraph, area)

    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::One(StateValue::Usize(self.top_line))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Scroll(Direction::Up) => {
                self.top_line = self.top_line.saturating_sub(1);
                CmdResult::None
            }
            Cmd::Scroll(Direction::Down) => {
                self.top_line = self.top_line.saturating_add(1);
                CmdResult::None
            }
            _ => CmdResult::None,
        }
    }
}

impl Component<UiMessage, UiMessage> for TextView {
    fn on(&mut self, ev: Event<UiMessage>) -> Option<UiMessage> {
        match ev {
            Event::Keyboard(key_event) => {
                match key_event.code {
                    Key::Up => {
                        self.top_line = self.top_line.saturating_sub(1);
                        Some(UiMessage::Redraw)
                    }
                    Key::Down =>  {
                        self.top_line = self.top_line.saturating_add(1);
                        Some(UiMessage::Redraw)
                    }
                    _ => None
                }
            }
            _ => None
        }
    }
}