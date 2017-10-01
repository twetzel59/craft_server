//! This module handles loading of nicknames from the nickname file.

use std::collections::HashMap;
//use std::io::Read;
use std::fs::{File, OpenOptions};
use std::net::IpAddr;

const FILE: &str = "nicks.txt";

/// Manages the nickname file.
pub struct NickManager {
    map: HashMap<IpAddr, String>,
    file: File,
}

impl NickManager {
    /// Creates a new NickManager.
    /// # Note
    /// The nicknames will be loaded from the nickname storage file.
    /// If the file doesn't exist, it will be created.
    /// # Panics
    /// This function will panic if it can't create or open the nicks file.
    pub fn new() -> NickManager {
        NickManager {
            map: HashMap::new(),
            file: OpenOptions::new()
                    .read(true)
                    .append(true)
                    .create(true)
                    .open(FILE)
                    .unwrap(),
        }
    }

    /*pub fn get(&self, id: Id) -> &str {

    }*/
}
