//! This module is the primary place for the server's core components.

use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use client;
use commands::CommandHandler;
use event::{Event, IdEvent, PositionEvent, TalkEvent};
use nick::NickManager;

/// The core server wrapper.
///
/// Runs the show, working with the incoming connections and handling
/// server events.
pub struct Server {
    listener: TcpListener,
    clients: Arc<Mutex<HashMap<client::Id, client::Client>>>,
    current_id: client::Id,
    channel: (mpsc::Sender<IdEvent>, mpsc::Receiver<IdEvent>),
    nicks: NickManager,
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
            nicks: NickManager::new(),
        };

        s.listener();
    }

    fn listener(mut self) {
        EventThread::run(self.channel.1, self.clients.clone());

        for i in self.listener.incoming() {
            let stream = i.unwrap();

            if let Ok(c) = client::Client::run(stream, self.channel.0.clone(), self.current_id, "bob") {
                self.clients.lock().unwrap().insert(self.current_id, c);
            }

            //for x in clients {
            //    println!("client {}: {}", x.id(), x.alive());
            //}

            self.current_id += 1;
        }
    }

    /*
    fn server(rx: mpsc::Receiver<Event>, clients: Arc<Mutex<Vec<client::Client>>>) {
        thread::spawn(move || {
            loop {
                if let Ok(ev) = rx.recv() {
                    println!("{:?}", ev);

                    for c in clients.lock().unwrap().iter() {
                        println!("{:?}", c.id());
                    }
                }
            }
        });
    }
    */

    /*fn handle_position_event(ev: &PositionEvent) {

    }*/
}

struct EventThread {
    rx: mpsc::Receiver<IdEvent>,
    clients: Arc<Mutex<HashMap<client::Id, client::Client>>>,
    command: CommandHandler,
}

impl EventThread {
    fn run(rx: mpsc::Receiver<IdEvent>, clients: Arc<Mutex<HashMap<client::Id, client::Client>>>) {
        let command = CommandHandler::new(clients.clone());

        let e = EventThread {
            rx,
            clients,
            command,
        };

        e.event_thread();
    }

    fn event_thread(self) {
        thread::spawn(move || {
            loop {
                if let Ok(ev) = self.rx.recv() {
                    //println!("{:?}", ev);

                    //for c in self.clients.lock().unwrap().iter() {
                    //    println!("{:?}", c.id());
                    //}

                    match ev.event {
                        Event::Position(p) => {
                            println!("{:?}", p);
                            self.handle_position_event(ev.id, p);
                        },
                        Event::Talk(t) => {
                            if t.text.starts_with('/') {
                                self.command.handle_command(&t.text[1..]);
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
