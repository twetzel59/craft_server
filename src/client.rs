//! The `client` module contains all of the machinery for the server-side representation of
//! a client.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::mpsc::Sender;
use std::thread;
use event::{BlockEvent, ChunkRequestEvent, Event, IdEvent, PositionEvent,
            LightEvent, SignEvent, TalkEvent};
use server::ServerTime;
use world::{Block, chunked, Light, Sign};

/// A type representing the ID players are given to uniquely identify them on both the client
/// and the server side.
pub type Id = u32;

/// The concrete represntation of a network client.
pub struct Client {
    send_stream: TcpStream,
    addr: IpAddr,
    nick: String,
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
               other_clients: &mut HashMap<Id, Client>) -> Result<Client, ()> {
        println!("New client id: {}", id);

        let send_stream = stream.try_clone().unwrap();

        let mut version_buf: [u8; 4] = [0; 4];
        stream.read_exact(&mut version_buf).unwrap();

        let addr = stream.peer_addr().unwrap();

        if version_buf == [b'V', b',', b'1', b'\n'] {
            println!("{:?} joined.", addr.to_string());

            ClientThread::run(stream, addr, tx, id, &nick, daytime, other_clients);

            let c = Client {
                send_stream,
                addr: addr.ip(),
                nick,
                position: (0., 0., 0., 0., 0.),
            };

            return Ok(c);
        } else {
            println!("{:?} denied.", addr.to_string());

            return Err(());
        }
    }

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

    /// Tells the client that a block has changed.
    pub fn send_block(&mut self, ev: &BlockEvent) {
        self.broadcast_block(((ev.x, ev.y, ev.z), &Block(ev.w)), (chunked(ev.x), chunked(ev.z)));
    }

    /// Tells the client that a light has changed.
    pub fn send_light(&mut self, ev: &LightEvent) {
        self.broadcast_light(((ev.x, ev.y, ev.z), &Light(ev.w)), (chunked(ev.x), chunked(ev.z)));
    }

    /// Notifies the client that another client has left.
    pub fn send_disconnect(&mut self, other_id: Id) {
        let msg = format!("D,{}\n", other_id);
        //println!("will send: {}", msg);

        // TODO: What if the stream is now closed? Alert something that client is disconnected.
        let _ = self.send_stream.write_all(msg.as_bytes());
    }

    /// Notifies a client that a sign has changed in the world.
    pub fn send_sign(&mut self, ev: &SignEvent) {
        let msg = format!("S,{},{},{},{},{},{},{}\n",
                          chunked(ev.x),
                          chunked(ev.z),
                          ev.x,
                          ev.y,
                          ev.z,
                          ev.face,
                          ev.text);
        //println!("will send: {}", msg);

        // TODO: What if the stream is now closed? Alert something that client is disconnected.
        let _ = self.send_stream.write_all(msg.as_bytes());
    }

    /// Sends a chat message without an event.
    pub fn broadcast_talk(&mut self, text: &str) {
        let msg = format!("T,{}\n", text);
        //println!("will send: {}", msg);

        // TODO: What if the stream is now closed? Alert something that client is disconnected.
        let _ = self.send_stream.write_all(msg.as_bytes());
    }

    /// Sends another player's nickname to this client.
    pub fn broadcast_nick(&mut self, other_id: Id, nick: &str) {
        let msg = format!("N,{},{}\n", other_id, nick);
        //println!("will send: {}", msg);

        // TODO: What if the stream is now closed? Alert something that client is disconnected.
        let _ = self.send_stream.write_all(msg.as_bytes());
    }

    /// Sends a block change without an event.
    pub fn broadcast_block(&mut self, block: ((i32, i32, i32), &Block), pq: (i32, i32)) {
        // We are sending a block with B,p,q,x,y,z,w.
        let msg = format!("B,{},{},{},{},{},{}\n",
                          pq.0, pq.1,
                          (block.0).0.to_string(),
                          (block.0).1.to_string(),
                          (block.0).2.to_string(),
                          (block.1).0.to_string());
        //println!("will send: {}", msg);

        // TODO: What if the stream is now closed? Alert something that client is disconnected.
        let _ = self.send_stream.write_all(msg.as_bytes());
    }

    /// Informs a client that a chunk needs to be redrawn.
    pub fn broadcast_redraw(&mut self, chunk: (i32, i32)) {
        let msg = format!("R,{},{}\n", chunk.0, chunk.1);
        //println!("will send: {}", msg);

        // TODO: What if the stream is now closed? Alert something that client is disconnected.
        let _ = self.send_stream.write_all(msg.as_bytes());
    }

    /// Sends a sign update to the client.
    pub fn broadcast_sign(&mut self, global_pos: (i32, i32, i32), face: u8, sign: &Sign) {
        let msg = format!("S,{},{},{},{},{},{},{}\n",
                          chunked(global_pos.0), chunked(global_pos.2),
                          global_pos.0, global_pos.1, global_pos.2,
                          face,
                          sign.0);
        //println!("will send: {}", msg);

        // TODO: What if the stream is now closed? Alert something that client is disconnected.
        let _ = self.send_stream.write_all(msg.as_bytes());
    }

    /// Sends a light update to the client.
    pub fn broadcast_light(&mut self, light: ((i32, i32, i32), &Light), pq: (i32, i32)) {
        let msg = format!("L,{},{},{},{},{},{}\n",
                          pq.0, pq.1,
                          (light.0).0, (light.0).1, (light.0).2,
                          (light.1).0);

        // TODO: What if the stream is now closed? Alert something that client is disconnected.
        let _ = self.send_stream.write_all(msg.as_bytes());
    }
}

struct ClientThread {
    stream: TcpStream,
    addr: SocketAddr,
    tx: Sender<IdEvent>,
    id: Id,
}

impl ClientThread {
    fn run(stream: TcpStream,
           addr: SocketAddr,
           tx: Sender<IdEvent>,
           id: Id,
           nick: &str,
           daytime: ServerTime,
           other_clients: &mut HashMap<Id, Client>) {
        let mut c = ClientThread {
            stream,
            addr,
            tx,
            id,
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

                    for i in msg.lines() {
                        //println!("i: {}", i);
                        self.handle_message(i);
                    }
                } else {
                    break;
                }
            }

            self.tx.send(IdEvent { id: self.id, peer: self.addr, event: Event::Disconnected }).unwrap();

            println!("A client left.");
        });
    }

    fn send_first_messages(&mut self,
                          nick: &str,
                          daytime: ServerTime,
                          other_clients: &mut HashMap<Id, Client>) {
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

            // Tell the client what the others' nickanmes are.
            // N,id,name
            let _ = self.stream.write_all(format!("N,{},{}\n", i.0, i.1.nick()).as_bytes());

            // The the *other* clients what that this player exists.
            // Note that in the Craft client, a player is initialized client-side
            // upon receiving of the first position message with the player's ID.
            i.1.send_position(self.id, &PositionEvent { x: 0., y: 0., z: 0., rx: 0., ry: 0. });

            // Tell the *other* clients what this player's nickname is.
            // N,id,name
            i.1.broadcast_nick(self.id, nick);
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
        } else if msg.starts_with('B') {
            self.handle_block(payload);
        } else if msg.starts_with('C') {
            self.handle_chunk(payload);
        } else if msg.starts_with('S') {
            self.handle_sign(payload);
        } else if msg.starts_with('L') {
            self.handle_light(payload);
        }
    }

    fn handle_position(&self, payload: &str) {
        if let Ok(ev) = PositionEvent::new(payload) {
            self.tx.send(IdEvent { id: self.id, peer: self.addr, event: Event::Position(ev) }).unwrap();
        }
    }

    fn handle_talk(&self, payload: &str) {
        if let Ok(ev) = TalkEvent::new(payload) {
            self.tx.send(IdEvent { id: self.id, peer: self.addr, event: Event::Talk(ev) }).unwrap();
        }
    }

    fn handle_block(&self, payload: &str) {
        if let Ok(ev) = BlockEvent::new(payload) {
            self.tx.send(IdEvent { id: self.id, peer: self.addr, event: Event::Block(ev) }).unwrap();
        }
    }

    fn handle_chunk(&self, payload: &str) {
        if let Ok(ev) = ChunkRequestEvent::new(payload) {
            self.tx.send(IdEvent { id: self.id, peer: self.addr, event: Event::ChunkRequest(ev) }).unwrap();
        }
    }

    fn handle_sign(&self, payload: &str) {
        if let Ok(ev) = SignEvent::new(payload) {
            self.tx.send(IdEvent { id: self.id, peer: self.addr, event: Event::Sign(ev) }).unwrap();
        }
    }

    fn handle_light(&self, payload: &str) {
        if let Ok(ev) = LightEvent::new(payload) {
            self.tx.send(IdEvent { id: self.id, peer: self.addr, event: Event::Light(ev) }).unwrap();
        }
    }
}
