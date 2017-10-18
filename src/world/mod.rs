//! This module contains the necessary functionality for representing
//! the world, both on disk and in memory.

mod queries;

const FILE: &str = "world.db";

use std::collections::HashMap;
use sqlite::{self, Connection};

const CHUNK_SIZE: u8 = 32;

/// Type of block IDs.
#[derive(Debug)]
pub struct Block(pub u8);

/* /// A structure representing a sector of the world.
pub */
#[derive(Debug)]
struct Chunk {
    blocks: HashMap<(u8, u8, u8), Block>,
}

impl Chunk {
    fn new() -> Chunk {
        Chunk {
            blocks: HashMap::new(),
        }
    }

    fn set_block(&mut self, local_pos: (u8, u8, u8), block: Block) {
        self.blocks.insert(local_pos, block);
    }
}

/// Manages a world and the SQLite connection to persist it on disk.
pub struct World {
    conn: Connection,
    chunks: HashMap<(i32, i32), Chunk>,
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
            chunks: HashMap::new(),
        };

        w.initial_queries();

        println!("OK");

        w
    }

    /// Set a block in the world with the given global coordinates.
    pub fn set_block(&mut self, global_pos: (i32, i32, i32), block: Block) {
        // P and Q are chunk/sector x and z.
        let (p, q) = (Self::chunked(global_pos.0), Self::chunked(global_pos.2));

        //println!("(p, q): {}, {}", p, q);
        let local_pos = ((global_pos.0 - p * CHUNK_SIZE as i32) as u8,
                          global_pos.1 as u8,
                         (global_pos.2 - q * CHUNK_SIZE as i32) as u8);

        {
            let entry = self.chunks.entry((p, q));
            let chunk = entry.or_insert(Chunk::new());
            chunk.set_block(local_pos, block);
        }

        println!("entire world: {:?}", self.chunks);
        /*
        if let Some(c) = self.chunks.get_mut(&(p, q)) {
            println!("case 1");

            c.set_block(local_pos, block);
        } else {
            println!("case 2");

            let mut c = Chunk::new((p, q));
            c.set_block(local_pos, block);
            self.chunks.insert((p, q), c);
        }
        */
    }

    fn initial_queries(&self) {
        for i in &queries::INITIAL {
            self.conn.execute(i).unwrap();
        }
    }

    fn chunked(n: i32) -> i32 {
        (n as f32 / CHUNK_SIZE as f32).floor() as i32
    }
}
