use std::{error::Error, net::TcpStream, sync::mpsc::Sender};
use tungstenite::{stream::MaybeTlsStream, Message};

pub struct WebSocket {
    tx: Sender<String>,
    socket: tungstenite::WebSocket<MaybeTlsStream<TcpStream>>,
}

impl WebSocket {
    pub fn new(
        tx: Sender<String>,
        url: &str,
        on_open_message: Option<&str>,
    ) -> Result<Self, Box<dyn Error>> {
        let (mut socket, _response) = tungstenite::connect(url)?;

        if let Some(s) = on_open_message {
            let message = Message::text(s);
            socket.write_message(message)?;
        }

        Ok(Self { tx, socket })
    }

    pub fn read_loop(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match self.socket.read_message() {
                Err(e) => return Err(e.into()),
                Ok(m) => match m {
                    Message::Text(s) => {
                        self.tx.send(s)?;
                    }
                    Message::Binary(_) => (),
                    Message::Ping(_) => {
                        // println!("ping");
                        // self.socket.write_message(Message::Pong(vec![]))?;
                    }
                    Message::Pong(_) => (),
                    Message::Close(_) => {
                        let _ = self.socket.close(None);
                        return Ok(());
                    }
                    Message::Frame(_) => (),
                },
            }
        }
    }
}
