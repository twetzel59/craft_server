//! The `client` module contains all of the machinery for the server-side representation of
//! a client.

//use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::thread;
use event::{Event, IdEvent, PositionEvent};

/// A type representing the ID players are given to uniquely identify them on both the client
/// and the server side.
pub type Id = u32;

/// The concrete represntation of a network client.
pub struct Client {
    send_stream: TcpStream,
    //queue: VecDeque<u8>,
    id: Id,
}

impl Client {
    /// Launches a new client with its TCP stream and a unique ID.
    pub fn run(mut stream: TcpStream, tx: Sender<IdEvent>, id: Id) -> Result<Client, ()> {
        println!("New client id: {}", id);

        let send_stream = stream.try_clone().unwrap();

        let mut version_buf: [u8; 4] = [0; 4];
        stream.read_exact(&mut version_buf).unwrap();

        let addr_str = match stream.peer_addr() {
            Ok(addr) => addr.to_string(),
            Err(_) => "<unknown addr>".to_string(),
        };

        if version_buf == [b'V', b',', b'1', b'\n'] {
            println!("{:?} joined.", addr_str);

            let c = Client {
                send_stream,
                //queue: VecDeque::new(),
                id,
            };

            ClientThread::run(stream, tx, id);

            return Ok(c);
        } else {
            println!("{:?} denied.", addr_str);

            return Err(());
        }
    }

    /// Returns the ID of this client.
    pub fn id(&self) -> Id {
        self.id
    }

    /// Send another client's position
    pub fn send_position(&mut self, other_id: Id, ev: &PositionEvent) {
        //println!("should send {}'s position to: {}", another_id, self.id);

        // Tell the client the other player's id and position.
        // P,id,x,y,z,rx,ry

        let msg = format!("P,{},{},{},{},{},{}\n",
                          other_id.to_string(),
                          ev.x.to_string(),
                          ev.y.to_string(),
                          ev.z.to_string(),
                          ev.rx.to_string(),
                          ev.ry.to_string());

        //print!("will send: {}", msg);

        // TODO: What if the stream is now closed? Alert something that client is disconnected.
        let _ = self.send_stream.write_all(msg.as_bytes());
    }
}

struct ClientThread {
    stream: TcpStream,
    tx: Sender<IdEvent>,
    id: Id,
    alive: bool,
}

impl ClientThread {
    fn run(stream: TcpStream, tx: Sender<IdEvent>, id: Id) {
        let c = ClientThread {
            stream,
            tx,
            id,
            alive: true,
        };

        c.client_thread();
    }

    fn client_thread(mut self) {
        thread::spawn(move || {
            self.send_first_messages();

            const BUFFER_LEN: usize = 4096;

            loop {
                let mut buf: [u8; BUFFER_LEN] = [0; BUFFER_LEN];

                let n_read = self.stream.read(&mut buf).unwrap();

                if n_read > 0 {
                    let msg = String::from_utf8_lossy(&buf);

                    //println!("msg: {}", msg);
                    self.handle_message(&msg);
                } else {
                    self.alive = false;

                    break;
                }

                //println!("buf: {:?}\n\n", &buf[0..32]);
                //if buf[0] == b'P' {
                //
                //}

                //for i in 0..n_written {
                //    self.queue.push_back(buf[i]);
                //}

                //println!("qsize: {}", self.queue.len());
            }

            println!("A client left.");
        });
    }

    fn send_first_messages(&mut self) {
        let id = self.id.to_string();

        // Tell the client the ID it has and where spawn is.
        // U,id,x,y,z,rx,ry
        let _ = self.stream.write_all(format!("U,{},0,0,0,0,0\n", id).as_bytes());

        // Tell the client its nickname.
        // N,id,name
        let _ = self.stream.write_all(format!("N,{},player{}\n", id, id).as_bytes());
    }

    fn handle_message(&self, msg: &str) {
        assert!(msg.len() > 1);
        //println!("message: {}", msg);

        if msg.starts_with('P') {
            self.handle_position(&msg[2..]);
        }
    }

    fn handle_position(&self, payload: &str) {
        //println!("client {} payload: {}", self.id, payload);

        //println!("{:?}", PositionEvent::new(payload));

        if let Ok(ev) = PositionEvent::new(payload) {
            self.tx.send(IdEvent { sender: self.id, event: Event::Position(ev) }).unwrap();
        }
    }
}
