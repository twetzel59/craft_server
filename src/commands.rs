//! The `commands` module contains the majority of the mechanism for handling chat commands.

use std::sync::{Arc, Mutex};
use client;

/// Allows processing of chat commands.
pub struct CommandHandler {
    clients: Arc<Mutex<Vec<client::Client>>>,
}

impl CommandHandler {
    /// Creates a new CommandHandler, requiring access to the server's client list.
    pub fn new(clients: Arc<Mutex<Vec<client::Client>>>) -> CommandHandler {
        CommandHandler {
            clients,
        }
    }

    /// Handle an incoming command.
    ///
    /// # Arguments
    /// * **command**: The slice of the command contains the name and arguments, but no `/`.
    pub fn handle_command(&self, command: &str) {
        if command.starts_with("say") {
            self.handle_say(command);
        } else {
            println!("UNKNOWN COMMAND");
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
                i.broadcast_talk(&announcement);
            }
        } else {
            println!("INVALID USAGE");
        }
    }
}
