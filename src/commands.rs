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
    /// * **command**: The slice of the command the contains the name and arguments, but no `/`.
    pub fn handle_command(&self, command: &str) {
        if command.starts_with("say") {
            self.handle_say(command);
        } else {
            println!("UNKNOWN COMMAND");
        }
    }

    fn handle_say(&self, command: &str) {
        if command.split(' ').next().unwrap_or("") == "say" {
            println!("SAY COMMAND");
        } else {
            println!("INVALID USAGE");
        }
    }
}
