//! This module is the primary place for the server's core components.

use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use client;
use commands::CommandHandler;
use event::{BlockEvent, ChunkRequestEvent, Event, IdEvent, PositionEvent, TalkEvent};
use nick::NickManager;
use world::{Block, World};

pub const DAY_LENGTH: u32 = 600;

/// The core server wrapper.
///
/// Runs the show, working with the incoming connections and handling
/// server events.
pub struct Server {
    listener: TcpListener,
    clients: Arc<Mutex<HashMap<client::Id, client::Client>>>,
    current_id: client::Id,
    disconnects: (mpsc::Sender<client::Id>, mpsc::Receiver<client::Id>),
    channel: (mpsc::Sender<IdEvent>, mpsc::Receiver<IdEvent>),
    nicks: Arc<Mutex<NickManager>>,
    daytime: ServerTime,
    world: World,
}

impl Server {
    /// Creates a new server and launches it. The server socket will be bound,
    /// and the server listener and event threads will start immediately.
    pub fn run() {
        let s = Server {
            listener: TcpListener::bind("0.0.0.0:4080").unwrap(),
            clients: Arc::new(Mutex::new(HashMap::new())),
            current_id: 1,
            disconnects: mpsc::channel(),
            channel: mpsc::channel(),
            nicks: Arc::new(Mutex::new(NickManager::new())),
            daytime: ServerTime {
                        from: Instant::now(),
                        offset: Duration::new(DAY_LENGTH as u64 / 2, 0),
            },
            world: World::new(),
        };

        s.listener();
    }

    fn listener(mut self) {
        EventThread::run(self.channel.1,
                         self.clients.clone(),
                         self.disconnects.0,
                         self.world,
                         self.nicks.clone());

        //let mut all_positions = Vec::new();
        for i in self.listener.incoming() {
            let stream = i.unwrap();

            let id;
            if let Ok(d) = self.disconnects.1.try_recv() {
                id = d;
            } else {
                id = self.current_id;
                self.current_id += 1;
            }

            let nick = match self.nicks.lock().unwrap().get(&stream.peer_addr().unwrap().ip()) {
                Some(s) => s.to_string(),
                None => "guest".to_string() + &id.to_string(),
            };

            let mut clients = self.clients.lock().unwrap();

            //all_positions.clear();
            //for i in clients.iter() {
            //    all_positions.push((*i.0, i.1.position()));
            //}
            //println!("all_positions: {:?}", all_positions);

            if let Ok(c) = client::Client::run(stream,
                                               self.channel.0.clone(),
                                               id,
                                               nick,
                                               self.daytime,
                                               &mut clients) {
                clients.insert(id, c);
            }

            //for x in clients {
            //    println!("client {}: {}", x.id(), x.alive());
            //}
        }
    }
}

struct EventThread {
    rx: mpsc::Receiver<IdEvent>,
    clients: Arc<Mutex<HashMap<client::Id, client::Client>>>,
    disconnects: mpsc::Sender<client::Id>,
    world: World,
    //nicks: Arc<Mutex<NickManager>>,
    command: CommandHandler,
}

impl EventThread {
    fn run(rx: mpsc::Receiver<IdEvent>,
           clients: Arc<Mutex<HashMap<client::Id, client::Client>>>,
           disconnects: mpsc::Sender<client::Id>,
           world: World,
           nicks: Arc<Mutex<NickManager>>) {
        let command = CommandHandler::new(clients.clone(), nicks);

        let e = EventThread {
            rx,
            clients,
            disconnects,
            world,
            //nicks,
            command,
        };

        e.event_thread();
    }

    fn event_thread(mut self) {
        thread::spawn(move || {
            loop {
                if let Ok(ev) = self.rx.recv() {
                    //println!("{:?}", ev);

                    //for c in self.clients.lock().unwrap().iter() {
                    //    println!("{:?}", c.id());
                    //}

                    match ev.event {
                        Event::Disconnected => {
                            self.handle_disconnect_event(ev.id);
                        },
                        Event::Position(p) => {
                            println!("{:?}", p);
                            self.handle_position_event(ev.id, p);
                        },
                        Event::Talk(t) => {
                            if t.text.starts_with('/') {
                                self.command.handle_command(ev.id, &t.text[1..]);
                            } else {
                                println!("CHAT: {} {}", ev.peer, t.text.lines().next().unwrap_or(""));
                                self.handle_talk_event(ev.id, t);
                            }
                        },
                        Event::Block(b) => {
                            println!("{:?}", b);
                            self.handle_block_event(ev.id, b);
                        }
                        Event::ChunkRequest(c) => {
                            println!("{:?}", c);
                            self.handle_chunk_event(ev.id, c);
                        }
                    }
                }
            }
        });
    }

    fn handle_disconnect_event(&mut self, id: client::Id) {
        let mut clients = self.clients.lock().unwrap();

        let msg = match clients.get(&id) {
            Some(client) => Some(client.nick().to_string() + " has left the game"),
            None => None,
        };

        clients.remove(&id);

        self.disconnects.send(id).unwrap();

        for i in clients.iter_mut() {
            if *i.0 != id {
                i.1.send_disconnect(id);

                if let Some(ref m) = msg {
                    i.1.broadcast_talk(&m);
                }
            }
        }
    }

    fn handle_position_event(&self, id: client::Id, ev: PositionEvent) {
        for i in self.clients.lock().unwrap().iter_mut() {
            if *i.0 != id {
                i.1.send_position(id, &ev);
            } else {
                i.1.set_position((ev.x, ev.y, ev.z, ev.rx, ev.ry));
            }
        }
    }

    fn handle_talk_event(&self, id: client::Id, mut ev: TalkEvent) {
        let mut clients = self.clients.lock().unwrap();

        ev.text = format!("{}> {}", clients.get(&id).unwrap().nick(), ev.text);

        for i in clients.iter_mut() {
            i.1.send_talk(&ev);
        }
    }

    fn handle_block_event(&mut self, id: client::Id, ev: BlockEvent) {
        use world::chunked;

        self.world.set_block((ev.x, ev.y, ev.z), Block(ev.w));

        for i in self.clients.lock().unwrap().iter_mut() {
            if *i.0 != id {
                i.1.send_block(&ev);
                i.1.broadcast_redraw((chunked(ev.x), chunked(ev.z)));
            }
        }
    }

    fn handle_chunk_event(&self, id: client::Id, ev: ChunkRequestEvent) {
        use world::CHUNK_SIZE;

        let mut clients = self.clients.lock().unwrap();

        if let Some(c) = clients.get_mut(&id) {
            if let Some(it) = self.world.blocks_in_chunk((ev.p, ev.q)) {
                for (xyz, w) in it {
                    println!("BLOCK: {}, {}, {}: {:?}", xyz.0, xyz.1, xyz.2, w);

                    // We need the absolute position in the world.
                    // Y axis is not divided into chunks.
                    let xyz = (xyz.0 as i32 + (ev.p * CHUNK_SIZE as i32),
                               xyz.1 as i32,
                               xyz.2 as i32 + (ev.q * CHUNK_SIZE as i32));
                    c.broadcast_block((xyz, w));
                }

                c.broadcast_redraw((ev.p, ev.q));
            }
        }
    }
}

/// Stores the data needed to find the game time of day.
#[derive(Copy, Clone)]
pub struct ServerTime {
    /// The actual time the game time was last set.
    pub from: Instant,

    /// The point to start counting from in seconds.
    pub offset: Duration,
}

impl ServerTime {
    pub fn time(&self) -> f32 {
        let duration = Instant::now() - self.from + self.offset;
        duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9
    }
}
