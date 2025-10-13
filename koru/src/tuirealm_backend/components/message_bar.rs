use tuirealm::{AttrValue, Attribute, Component, Event, Frame, MockComponent, Props, State};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::prelude::Text;
use tuirealm::ratatui::widgets::Paragraph;
use crate::tuirealm_backend::UiMessage;

pub struct MessageBar {
    props: Props,
}

impl MessageBar {
    pub fn new() -> Self {
        let mut props = Props::default();
        props.set(Attribute::Text, AttrValue::String(String::new()));
        MessageBar { props }
    }
}

impl MockComponent for MessageBar {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let Some(AttrValue::String(content)) = self.query(Attribute::Text) else {
            return;
        };

        let text = Text::from(content);
        let p = Paragraph::new(text);
        frame.render_widget(p, area);
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

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<UiMessage, UiMessage> for MessageBar {
    fn on(&mut self, ev: Event<UiMessage>) -> Option<UiMessage> {
        None
    }
}