//! This module handles loading of nicknames from the nickname file.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
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
        let mut m = NickManager {
            map: HashMap::new(),
            file: OpenOptions::new()
                    .read(true)
                    .append(true)
                    .create(true)
                    .open(FILE)
                    .unwrap(),
        };

        m.load();

        m
    }

    /// Get the nickname associated with an IP address.
    /// # Return value
    /// If the nickname was found, the nickname is returned.
    /// Otherwise, None is returned.
    pub fn get(&self, ip: &IpAddr) -> Option<&str> {
        match self.map.get(ip) {
            Some(ref s) => Some(s),
            None => None,
        }
    }

    /// Set the nickname for an IP address.
    pub fn set(&mut self, ip: &IpAddr, nick: &str) {
        self.map.insert(ip.clone(), nick.to_string());

        self.save();
    }

    fn load(&mut self) {
        for i in BufReader::new(self.file.try_clone().unwrap()).lines() {
            let i = i.unwrap();

            if i.split('=').count() != 2 {
                panic!("Can't read {}", FILE);
            }

            let mut pieces: Vec<&str> = i.split(|c: char| c == '=' || c.is_whitespace()).collect();
            pieces.retain(|p| *p != "");

            if pieces.len() != 2 {
                panic!("Can't read {}", FILE);
            }

            //println!("pieces: {:?}", pieces);

            let ip = pieces[0].parse();
            assert!(ip.is_ok(), "Can't read {}", FILE);

            self.map.insert(ip.unwrap(), pieces[1].to_string());
        }

        println!("nickname map: {:?}", self.map);
    }

    fn save(&mut self) {
        self.file.set_len(0).unwrap();

        for i in &self.map {
            self.file.write_fmt(format_args!("{} = {}\n", i.0, i.1.to_string())).unwrap();
        }

        self.file.flush().unwrap();
    }
}
