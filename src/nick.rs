//! This module handles loading of nicknames from the nickname file.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::fs::{File, OpenOptions};
use std::net::IpAddr;
//use std::str::FromStr;

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

    fn load(&mut self) {
        /*
        let content: String = self.file.by_ref()
                                       .bytes()
                                       .take_while(|b| match *b {
                                           Ok(b) => b != b'\n',
                                           Err(_) => false,
                                       })
                                       .map(|b| b.unwrap() as char)
                                       .collect();
        println!("content: {}", content);
        */

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
}
