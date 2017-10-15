//! This module contains the necessary functionality for representing
//! the world, both on disk and in memory.

mod queries;

const FILE: &str = "world.db";

use sqlite::{self, Connection};

/// Manages a world and the SQLite connection to persist it on disk.
pub struct World {
    conn: Connection,
}

impl World {
    /// Create a new world manager. The SQLite database will be created
    /// or opened.
    /// # Panics
    /// This function panics if the SQLite connection fails or the necessary
    /// initial queries can not be performed successfully.
    pub fn new() -> World {
        print!("Loading world... ");

        let conn = sqlite::open(FILE).unwrap();

        let w = World {
            conn,
        };

        w.initial_queries();

        println!("OK");

        w
    }

    fn initial_queries(&self) {
        for i in &queries::INITIAL {
            self.conn.execute(i).unwrap();
        }
    }
}
