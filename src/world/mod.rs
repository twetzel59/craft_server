//! This module contains the necessary functionality for representing
//! the world, both on disk and in memory.

mod queries;

const FILE: &str = "world.db";

use std::collections::hash_map;
use std::collections::HashMap;
use sqlite::{self, Connection};

/// The square X and Z dimensions of a world sector.
pub const CHUNK_SIZE: u8 = 32;

/// Type of block IDs.
#[derive(Debug)]
pub struct Block(pub i8);

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

impl<'a> IntoIterator for &'a Chunk {
    type Item = (&'a (u8, u8, u8), &'a Block);
    type IntoIter = hash_map::Iter<'a, (u8, u8, u8), Block>;

    fn into_iter(self) -> hash_map::Iter<'a, (u8, u8, u8), Block> {
        self.blocks.iter()
    }
}

struct ChunkManager {
    chunks: HashMap<(i32, i32), Chunk>,
}

impl ChunkManager {
    fn new() -> ChunkManager {
        ChunkManager {
            chunks: HashMap::new(),
        }
    }

    fn set_block(&mut self, global_pos: (i32, i32, i32), pq: (i32, i32), block: Block) {
        // P and Q are chunk/sector x and z.
        //let (p, q) = (chunked(global_pos.0), chunked(global_pos.2));

        //println!("(p, q): {}, {}", p, q);
        let local_pos = ((global_pos.0 - pq.0 * CHUNK_SIZE as i32 + 1) as u8,
                          global_pos.1 as u8,
                         (global_pos.2 - pq.1 * CHUNK_SIZE as i32 + 1) as u8);
        //println!("local_pos: {:?}", local_pos);
        //println!("test: {}", (global_pos.0 - pq.0 * CHUNK_SIZE as i32));

        let entry = self.chunks.entry(pq);
        let chunk = entry.or_insert(Chunk::new());
        chunk.set_block(local_pos, block);

        //println!("entire world: {:?}", self.chunks);
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

        /*
        print!("chunks: ");
        for i in self.chunks.keys() {
            print!("{:?}, ", i);
        }
        println!();
        */
    }

    fn get(&self, pq: (i32, i32)) -> Option<&Chunk> {
        self.chunks.get(&pq)
    }
}

/// Manages a world and the SQLite connection to persist it on disk.
pub struct World {
    conn: Connection,
    chunk_mgr: ChunkManager,
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

        let mut w = World {
            conn,
            chunk_mgr: ChunkManager::new(),
        };

        w.initial_queries();
        w.load_blocks();

        println!("OK");

        w
    }

    /// Set a block in the world with the given global coordinates. The chunk is set
    /// manually to avoid troubles with chunk borders.
    pub fn set_block(&mut self, global_pos: (i32, i32, i32), pq: (i32, i32), block: Block) {
        self.chunk_mgr.set_block(global_pos, pq, block);
    }

    pub fn blocks_in_chunk(&self, chunk: (i32, i32)) -> Option<hash_map::Iter<(u8, u8, u8), Block>> {
        match self.chunk_mgr.get(chunk) {
            Some(c) => Some(c.into_iter()),
            None => None,
        }
    }

    fn initial_queries(&self) {
        self.conn.execute(queries::INITIAL).unwrap();
    }

    fn load_blocks(&mut self) {
        let mut cursor = self.conn.prepare(queries::LOAD_BLOCKS).unwrap().cursor();

        while let Some(record) = cursor.next().unwrap() {
            let (xyz, w) = ((record[0].as_integer().unwrap() as i32,
                             record[1].as_integer().unwrap() as i32,
                             record[2].as_integer().unwrap() as i32),
                             record[3].as_integer().unwrap() as i8);

            //println!("values: ({}, {}, {}): {}", x, y, z, w);
            self.chunk_mgr.set_block(xyz, (chunked(xyz.0), chunked(xyz.1)), Block(w));
        }
    }
}

/// Return the chunk that a block falls in on one axis.
pub fn chunked(n: i32) -> i32 {
    (n as f32 / CHUNK_SIZE as f32).floor() as i32
}
