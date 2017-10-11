//! The `client` module contains all of the machinery for the server-side representation of
//! a client.

use std::collections::HashMap;
//use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
//use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::mpsc::Sender;
use std::thread;
use event::{Event, IdEvent, PositionEvent, TalkEvent};
use server::ServerTime;

/// A type representing the ID players are given to uniquely identify them on both the client
/// and the server side.
pub type Id = u32;

/// The concrete represntation of a network client.
pub struct Client {
    send_stream: TcpStream,
    //queue: VecDeque<u8>,
    //id: Id,
    addr: IpAddr,
    nick: String,
    //thread_death: Receiver<()>,
    //alive: bool,
    position: (f32, f32, f32, f32, f32),
}

impl Client {
    /// Launches a new client with its TCP stream, a unique ID, and its nickname.
    /// Also needed is the current server time and player transforms.
    pub fn run(mut stream: TcpStream,
               tx: Sender<IdEvent>,
               id: Id,
               nick: String,
               daytime: ServerTime,
               other_clients: &HashMap<Id, Client>) -> Result<Client, ()> {
        println!("New client id: {}", id);

        let send_stream = stream.try_clone().unwrap();

        let mut version_buf: [u8; 4] = [0; 4];
        stream.read_exact(&mut version_buf).unwrap();

        /*
        let addr_str = match stream.peer_addr() {
            Ok(addr) => addr.to_string(),
            Err(_) => "<unknown addr>".to_string(),
        };
        */

        let addr = stream.peer_addr().unwrap();

        if version_buf == [b'V', b',', b'1', b'\n'] {
            println!("{:?} joined.", addr.to_string());

            //let (death_notifier, thread_death) = mpsc::channel();

            //ClientThread::run(stream, addr, tx, id, &nick, death_notifier);
            ClientThread::run(stream, addr, tx, id, &nick, daytime, &other_clients);

            let c = Client {
                send_stream,
                //queue: VecDeque::new(),
                //id,
                addr: addr.ip(),
                nick,
                //thread_death,
                //alive: true,
                position: (0., 0., 0., 0., 0.),
            };

            return Ok(c);
        } else {
            println!("{:?} denied.", addr.to_string());

            return Err(());
        }
    }

    /*
    /// Returns the ID of this client.
    pub fn id(&self) -> Id {
        self.id
    }
    */

    /// Returns this client's nickname.
    pub fn nick(&self) -> &str {
        &self.nick
    }

    /// Sets this client's nickname.
    pub fn set_nick(&mut self, nick: &str) {
        self.nick = nick.to_string();
    }

    /// Returns the IP address of this peer.
    pub fn addr(&self) -> &IpAddr {
        &self.addr
    }

    /// Returns the player's transform, in the format
    /// (x, y, z, rx, ry)
    pub fn position(&self) -> (f32, f32, f32, f32, f32) {
        self.position
    }

    /// Sets the player's transform **server-side**
    pub fn set_position(&mut self, position: (f32, f32, f32, f32, f32)) {
        self.position = position;
    }

    /*
    /// Determine if the client is alive.
    /// # Note
    /// The `self` reference is mutable here, because the object state will save
    /// the last check to avoid checking the client thread for its status again.
    pub fn alive(&mut self) -> bool {
        if self.alive {
            // If the player left, the client thread will have died
            // since the object state was updated!

            match self.thread_death.try_recv() {
                Ok(()) => self.alive = false,
                Err(e) => match e {
                    TryRecvError::Disconnected => self.alive = false,
                    _ => {},
                },
            };

            self.alive
        } else {
            false
        }
    }
    */

    /// Sends another client's position.
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

    /// Sends a chat message.
    pub fn send_talk(&mut self, ev: &TalkEvent) {
        self.broadcast_talk(&ev.text);
    }

    /// Sends a chat message without an event.
    pub fn broadcast_talk(&mut self, text: &str) {
        let msg = format!("T,{}\n", text);
        //println!("will send: {}", msg);

        // TODO: What if the stream is now closed? Alert something that client is disconnected.
        let _ = self.send_stream.write_all(msg.as_bytes());
    }
}

struct ClientThread {
    stream: TcpStream,
    addr: SocketAddr,
    tx: Sender<IdEvent>,
    id: Id,
    //death_notifier: Sender<()>,
}

impl ClientThread {
    fn run(stream: TcpStream,
           addr: SocketAddr,
           tx: Sender<IdEvent>,
           id: Id,
           nick: &str,
           daytime: ServerTime,
           other_clients: &HashMap<Id, Client>) {
           //all_positions: &Vec<(Id, (f32, f32, f32, f32, f32))>) {
           //death_notifier: Sender<()>) {
        let mut c = ClientThread {
            stream,
            addr,
            tx,
            id,
            //death_notifier,
        };

        c.send_first_messages(nick, daytime, other_clients);
        c.client_thread();
    }

    fn client_thread(mut self) {
        thread::spawn(move || {
            const BUFFER_LEN: usize = 4096;

            loop {
                let mut buf: [u8; BUFFER_LEN] = [0; BUFFER_LEN];

                let n_read = self.stream.read(&mut buf).unwrap();

                if n_read > 0 {
                    let msg = String::from_utf8_lossy(&buf);

                    //println!("msg: {}", msg);
                    self.handle_message(&msg);
                } else {
                    break;
                }
            }

            self.tx.send(IdEvent { id: self.id, peer: self.addr, event: Event::Disconnected }).unwrap();
            //self.death_notifier.send(()).unwrap();

            println!("A client left.");
        });
    }

    fn send_first_messages(&mut self,
                          nick: &str,
                          daytime: ServerTime,
                          other_clients: &HashMap<Id, Client>) {
                          //all_positions: &Vec<(Id, (f32, f32, f32, f32, f32))>) {
        use server::DAY_LENGTH;

        let id = self.id.to_string();

        // Tell the client the ID it has and where spawn is.
        // U,id,x,y,z,rx,ry
        let _ = self.stream.write_all(format!("U,{},0,0,0,0,0\n", id).as_bytes());

        // Tell the client the current server time.
        // E,time,day_length
        let _ = self.stream.write_all(format!("E,{},{}\n",
                                              daytime.time(),
                                              DAY_LENGTH).as_bytes());


        for i in other_clients {
            let transform = i.1.position();

            // Tell the client where other players are.
            // P,id,x,y,z,rx,ry
            let _ = self.stream.write_all(format!("P,{},{},{},{},{},{}\n",
                                                  i.0,
                                                  transform.0,
                                                  transform.1,
                                                  transform.2,
                                                  transform.3,
                                                  transform.4).as_bytes());
        }

        // Tell the client its nickname.
        // N,id,name
        let _ = self.stream.write_all(format!("N,{},{}\n", id, nick).as_bytes());
    }

    fn handle_message(&self, msg: &str) {
        assert!(msg.len() > 1);
        //println!("message: {}", msg);

        let payload = &msg[2..];
        if msg.starts_with('P') {
            self.handle_position(payload);
        } else if msg.starts_with('T') {
            self.handle_talk(payload);
        }
    }

    fn handle_position(&self, payload: &str) {
        //println!("client {} payload: {}", self.id, payload);

        //println!("{:?}", PositionEvent::new(payload));

        if let Ok(ev) = PositionEvent::new(payload) {
            self.tx.send(IdEvent { id: self.id, peer: self.addr, event: Event::Position(ev) }).unwrap();
        }
    }

    fn handle_talk(&self, payload: &str) {
        if let Ok(ev) = TalkEvent::new(payload) {
            self.tx.send(IdEvent { id: self.id, peer: self.addr, event: Event::Talk(ev) }).unwrap();
        }
    }
}
