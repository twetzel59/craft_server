//! This module is the primary place for the server's core components.

use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use client;
use commands::CommandHandler;
use event::{Event, IdEvent, PositionEvent, TalkEvent};
use nick::NickManager;

pub const DAY_LENGTH: u32 = 600;

/// The core server wrapper.
///
/// Runs the show, working with the incoming connections and handling
/// server events.
pub struct Server {
    listener: TcpListener,
    clients: Arc<Mutex<HashMap<client::Id, client::Client>>>,
    current_id: client::Id,
    channel: (mpsc::Sender<IdEvent>, mpsc::Receiver<IdEvent>),
    nicks: Arc<Mutex<NickManager>>,
    daytime: ServerTime,
}

impl Server {
    /// Creates a new server and launches it. The server socket will be bound,
    /// and the server listener and event threads will start immediately.
    pub fn run() {
        let s = Server {
            listener: TcpListener::bind("127.0.0.1:4080").unwrap(),
            clients: Arc::new(Mutex::new(HashMap::new())),
            current_id: 1,
            channel: mpsc::channel(),
            nicks: Arc::new(Mutex::new(NickManager::new())),
            daytime: ServerTime {
                        from: Instant::now(),
                        offset: Duration::new(DAY_LENGTH as u64 / 2, 0)
            },
        };

        s.listener();
    }

    fn listener(mut self) {
        EventThread::run(self.channel.1, self.clients.clone(), self.nicks.clone());

        for i in self.listener.incoming() {
            let stream = i.unwrap();

            let nick = match self.nicks.lock().unwrap().get(&stream.peer_addr().unwrap().ip()) {
                Some(s) => s.to_string(),
                None => "player".to_string() + &self.current_id.to_string(),
            };

            if let Ok(c) = client::Client::run(stream,
                                               self.channel.0.clone(),
                                               self.current_id,
                                               nick,
                                               self.daytime) {
                self.clients.lock().unwrap().insert(self.current_id, c);
            }

            //for x in clients {
            //    println!("client {}: {}", x.id(), x.alive());
            //}

            self.current_id += 1;
        }
    }
}

struct EventThread {
    rx: mpsc::Receiver<IdEvent>,
    clients: Arc<Mutex<HashMap<client::Id, client::Client>>>,
    //nicks: Arc<Mutex<NickManager>>,
    command: CommandHandler,
}

impl EventThread {
    fn run(rx: mpsc::Receiver<IdEvent>,
           clients: Arc<Mutex<HashMap<client::Id, client::Client>>>,
           nicks: Arc<Mutex<NickManager>>) {
        let command = CommandHandler::new(clients.clone(), nicks);

        let e = EventThread {
            rx,
            clients,
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
                    }
                }
            }
        });
    }

    fn handle_disconnect_event(&mut self, id: client::Id) {
        let mut clients = self.clients.lock().unwrap();

        clients.remove(&id);
    }

    fn handle_position_event(&self, id: client::Id, ev: PositionEvent) {
        for i in self.clients.lock().unwrap().iter_mut() {
            if *i.0 != id {
                i.1.send_position(id, &ev);
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
