use std::error::Error;
use std::io::Stdout;
use std::sync::mpsc::{Receiver, Sender};
use tuirealm::{Application, AttrValue, Attribute, EventListenerCfg, Frame, MockComponent, NoUserEvent, PollStrategy, State, Update};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::props::Style;
use tuirealm::ratatui::backend::CrosstermBackend;
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::style::Styled;
use tuirealm::ratatui::widgets::Paragraph;
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalBridge};
use koru_core::kernel::broker::BrokerClient;
use koru_core::kernel::client::{ClientConnectingMessage, ClientConnectingResponse};
use koru_core::styled_text::StyledFile;
use crate::common::UiMessage;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum Id {

}

struct App {
    quit: bool,
    redraw: bool,
    terminal: TerminalBridge<CrosstermTerminalAdapter>,
    broker_client: BrokerClient,
    session_address: Option<usize>,
    text: StyledFile,
}

impl App {
    pub fn view(&mut self, app: &mut Application<Id, UiMessage, NoUserEvent>) {
        self.terminal.draw(|frame| {
            frame.render_widget(Paragraph::new("Hello, Koru!"), frame.area());
        }).unwrap();
    }
}

impl Update<UiMessage> for App {
    fn update(&mut self, msg: Option<UiMessage>) -> Option<UiMessage> {
        None
    }
}

fn init_app() -> Application<Id, UiMessage, NoUserEvent> {
    let mut app = Application::init(
        EventListenerCfg::default()
    );
    app
}

pub fn real_main(
    client_connector: Sender<ClientConnectingMessage>,
    client_receiver: Receiver<ClientConnectingResponse>,
) -> Result<(), Box<dyn Error>> {

    let mut application = init_app();
    client_connector.send(ClientConnectingMessage::RequestLocalConnection).unwrap();
    let client = client_receiver.recv().unwrap();
    let client = match client {
        ClientConnectingResponse::Connection {
            client
        } => {
            client
        }
        _ => {
            return Err(Box::from(String::from("Unable to connect to Koru")));
        }
    };
    
    let mut app = App {
        quit: false,
        redraw: true,
        terminal: TerminalBridge::new_crossterm().unwrap(),
        broker_client: client,
        session_address: None,
        text: StyledFile::new()
    };

    let _ = app.terminal.enter_alternate_screen()?;
    let _ = app.terminal.enable_raw_mode()?;

    while !app.quit {
        match application.tick(PollStrategy::Once) {
            Err(err) => {
                app.terminal.disable_raw_mode()?;
                app.terminal.leave_alternate_screen()?;
            }
            Ok(messages) => {
                for message in messages {
                    
                }
            }
        }
        if app.redraw {
            let _ = app.view(&mut application);
            app.redraw = false;
        }
    }

    app.terminal.disable_raw_mode()?;
    app.terminal.leave_alternate_screen()?;
    Ok(())
}