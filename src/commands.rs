//! The `commands` module contains the majority of the mechanism for handling chat commands.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use client;
use nick::NickManager;

/// Allows processing of chat commands.
pub struct CommandHandler {
    clients: Arc<Mutex<HashMap<client::Id, client::Client>>>,
    nicks: Arc<Mutex<NickManager>>,
}

impl CommandHandler {
    /// Creates a new CommandHandler, requiring access to the server's client list and nicknames.
    pub fn new(clients: Arc<Mutex<HashMap<client::Id, client::Client>>>,
               nicks: Arc<Mutex<NickManager>>) -> CommandHandler {
        CommandHandler {
            clients,
            nicks,
        }
    }

    /// Handle an incoming command.
    ///
    /// # Arguments
    /// * **command**: The slice of the command contains the name and arguments, but no `/`.
    pub fn handle_command(&mut self, id: client::Id, command: &str) {
        if command.starts_with("say ") {
            self.handle_say(command);
        } else if command.starts_with("nick ") {
            self.handle_nick(id, command);
        } else {
            println!("UNKNOWN COMMAND OR INVALID USAGE");
        }
    }

    fn handle_say(&self, command: &str) {
        let mut it = command.split(' ').peekable();

        if it.next().unwrap_or("") == "say" {
            let preserve_whitespace = if *it.peek().unwrap_or(&"") == "-w" {
                println!("SAY COMMAND (PRESERVE WHITESPACE)");

                let _ = it.next();

                true
            } else {
                println!("SAY COMMAND");

                false
            };

            // TODO: Efficiency?
            let announcement: String = it
                    .filter(|s| preserve_whitespace || *s != "")
                    .map(|s| (s.to_string() + " ").lines().next().unwrap_or("").to_string())
                    .collect();

            if announcement == "" || announcement == "-w" {
                println!("INVALID USAGE");
                return;
            }

            println!("announcement: {:?}", announcement);

            for i in self.clients.lock().unwrap().iter_mut() {
                i.1.broadcast_talk(&announcement);
            }
        } else {
            println!("INVALID USAGE");
        }
    }

    fn handle_nick(&mut self, id: client::Id, command: &str) {
        let mut it = command.split(' ');

        if it.next().unwrap_or("") == "nick" {
            let nick;

            if let Some(n) = it.next() {
                let mut n = n.trim();
                if n.find(|c: char| !c.is_whitespace() && !c.is_control()).is_none() {
                    println!("INVALID USAGE");
                    return;
                }

                n = n.trim_matches(|c: char| c.is_control());
                nick = n;

                if it.next().is_some() {
                    println!("INVALID USAGE");
                    return;
                }

                println!("new nick: {}", nick);

                let mut clients = self.clients.lock().unwrap();
                let msg;
                {
                    let c = clients.get_mut(&id).unwrap();

                    if c.nick() == nick {
                        println!("Client at {:?} tried to reset the current nick", c.addr());
                        return;
                    }

                    self.nicks.lock().unwrap().set(&c.addr(), nick);

                    msg = format!("{} is now known as: {}", c.nick(), nick);
                    c.set_nick(nick);
                }

                for i in clients.iter_mut() {
                    i.1.broadcast_talk(&msg);
                }
            }
        }
    }
}
