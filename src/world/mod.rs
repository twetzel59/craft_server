//! This module contains the necessary functionality for representing
//! the world, both on disk and in memory.

mod queries;

const FILE: &str = "world.db";

use std::collections::hash_map;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use sqlite::{self, Connection};

/// The square X and Z dimensions of a world sector.
pub const CHUNK_SIZE: u8 = 32;

/// Type of block IDs.
#[derive(Clone, Debug)]
pub struct Block(pub i8);

/// Type of Craft signs.
#[derive(Clone, Debug)]
pub struct Sign(pub String);

/* /// A structure representing a sector of the world.
pub */
#[derive(Debug)]
struct Chunk {
    blocks: HashMap<(u8, u8, u8), Block>,
    signs: HashMap<(i32, i32, i32, u8), Sign>,
}

impl Chunk {
    fn new() -> Chunk {
        Chunk {
            blocks: HashMap::new(),
            signs: HashMap::new(),
        }
    }

    fn set_block(&mut self, local_pos: (u8, u8, u8), block: Block) {
        self.blocks.insert(local_pos, block);
    }

    fn set_sign(&mut self, global_pos: (i32, i32, i32), face: u8, sign: Sign) {
        let key = (global_pos.0, global_pos.1, global_pos.2, face);

        self.signs.insert(key, sign);
    }

    fn signs(&self) -> hash_map::Iter<(i32, i32, i32, u8), Sign> {
        self.signs.iter()
    }
}

impl<'a> IntoIterator for &'a Chunk {
    type Item = (&'a (u8, u8, u8), &'a Block);
    type IntoIter = hash_map::Iter<'a, (u8, u8, u8), Block>;

    fn into_iter(self) -> Self::IntoIter {
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

    fn set_sign(&mut self, global_pos: (i32, i32, i32), pq: (i32, i32), face: u8, sign: Sign) {
        let entry = self.chunks.entry(pq);
        let chunk = entry.or_insert(Chunk::new());
        chunk.set_sign(global_pos, face, sign);

        //println!("all blocks and signs: {:?}", self.chunks);
    }

    fn get(&self, pq: (i32, i32)) -> Option<&Chunk> {
        self.chunks.get(&pq)
    }
}

/// Manages a world and the SQLite connection to persist it on disk.
pub struct World {
    chunk_mgr: ChunkManager,
    tx: mpsc::Sender<DatabaseCommand>,
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
        let channel = mpsc::channel();

        let mut w = World {
            chunk_mgr: ChunkManager::new(),
            tx: channel.0,
        };

        w.initial_queries(&conn);
        w.load_blocks(&conn);
        w.load_signs(&conn);

        println!("OK");

        DatabaseThread::run(conn, channel.1);

        w
    }

    /// Set a block in the world with the given global coordinates. The chunk is set
    /// manually to avoid troubles with chunk borders.
    pub fn set_block(&mut self, global_pos: (i32, i32, i32), pq: (i32, i32), block: Block) {
        self.chunk_mgr.set_block(global_pos, pq, block.clone());
        self.tx.send(DatabaseCommand::SetBlock(SetBlockCommand {
            xyz: global_pos,
            pq,
            block,
        })).unwrap();
    }

    /// Set a sign in the world using absolute world coordinates and chunk coordinates.
    pub fn set_sign(&mut self, global_pos: (i32, i32, i32), pq: (i32, i32), face: u8, sign: Sign) {
        self.chunk_mgr.set_sign(global_pos, pq, face, sign.clone());
        self.tx.send(DatabaseCommand::SetSign(SetSignCommand {
            xyz: global_pos,
            face,
            sign,
        })).unwrap();
    }

    /// Iterate over the blocks in the chunk with these (P, Q) (as in (X, Z)) coordinates.
    pub fn blocks_in_chunk(&self, chunk: (i32, i32)) -> Option<hash_map::Iter<(u8, u8, u8), Block>> {
        match self.chunk_mgr.get(chunk) {
            Some(c) => Some(c.into_iter()),
            None => None,
        }
    }

    /// Iterate over the signs in the chunk with these coordinates.
    pub fn signs_in_chunk(&self, chunk: (i32, i32)) -> Option<hash_map::Iter<(i32, i32, i32, u8), Sign>> {
        match self.chunk_mgr.get(chunk) {
            Some(c) => Some(c.signs()),
            None => None,
        }
    }

    fn initial_queries(&self, conn: &Connection) {
        conn.execute(queries::INITIAL).unwrap();
    }

    fn load_blocks(&mut self, conn: &Connection) {
        let mut cursor = conn.prepare(queries::LOAD_BLOCKS).unwrap().cursor();

        while let Some(record) = cursor.next().unwrap() {
            let (pq, xyz, w) = ((record[0].as_integer().unwrap() as i32,
                                 record[1].as_integer().unwrap() as i32),
                                (record[2].as_integer().unwrap() as i32,
                                 record[3].as_integer().unwrap() as i32,
                                 record[4].as_integer().unwrap() as i32),
                                 record[5].as_integer().unwrap() as i8);

            //println!("values: ({}, {}, {}): {}", x, y, z, w);
            self.chunk_mgr.set_block(xyz, pq, Block(w));
        }
    }

    fn load_signs(&mut self, conn: &Connection) {
        let mut cursor = conn.prepare(queries::LOAD_SIGNS).unwrap().cursor();

        while let Some(record) = cursor.next().unwrap() {
            let (pq, xyz, face, text) = ((record[0].as_integer().unwrap() as i32,
                                          record[1].as_integer().unwrap() as i32),
                                         (record[2].as_integer().unwrap() as i32,
                                          record[3].as_integer().unwrap() as i32,
                                          record[4].as_integer().unwrap() as i32),
                                          record[5].as_integer().unwrap() as u8,
                                          record[6].as_string().unwrap().to_string());

            self.chunk_mgr.set_sign(xyz, pq, face, Sign(text));
        }
    }
}

struct SetBlockCommand {
    pub xyz: (i32, i32, i32),
    pub pq: (i32, i32),
    pub block: Block,
}

struct SetSignCommand {
    pub xyz: (i32, i32, i32),
    pub face: u8,
    pub sign: Sign,
}

enum DatabaseCommand {
    SetBlock(SetBlockCommand),
    SetSign(SetSignCommand),
}

struct DatabaseThread {
    conn: Connection,
    rx: mpsc::Receiver<DatabaseCommand>,
}

impl DatabaseThread {
    fn run(conn: Connection, rx: mpsc::Receiver<DatabaseCommand>) {
        let d = DatabaseThread {
            conn,
            rx,
        };

        d.database_thread();
    }

    fn database_thread(self) {
        use std::time::Duration;

        thread::spawn(move || {
            loop {
                let mut changed = false;

                while let Ok(cmd) = self.rx.try_recv() {
                    changed = true;

                    match cmd {
                        DatabaseCommand::SetBlock(c) => self.handle_set_block(&c),
                        DatabaseCommand::SetSign(c) => self.handle_set_sign(&c),
                    }
                }

                if changed {
                    //self.conn.execute(queries::COMMIT).unwrap();
                    println!("Saved the world.");
                }

                thread::sleep(Duration::from_secs(5));
            }
        });
    }

    fn handle_set_block(&self, cmd: &SetBlockCommand) {
        let query = format!("{}({}, {}, {}, {}, {}, {});
                            {} p = {} AND q = {} AND x = {} AND y = {} AND z = {};",

                            queries::SET_BLOCK,
                            cmd.pq.0,
                            cmd.pq.1,
                            cmd.xyz.0,
                            cmd.xyz.1,
                            cmd.xyz.2,
                            cmd.block.0,

                            queries::DELETE_SIGN,
                            cmd.pq.0,
                            cmd.pq.1,
                            cmd.xyz.0,
                            cmd.xyz.1,
                            cmd.xyz.2);

        self.conn.execute(query).unwrap();
        //println!("{}", query);
    }

    fn handle_set_sign(&self, cmd: &SetSignCommand) {
        let pq = (chunked(cmd.xyz.0), chunked(cmd.xyz.2));

        let query = if cmd.sign.0 == "" {
            println!("delete!");

            format!("{} p = {} AND q = {} AND x = {} AND y = {} AND z = {} AND face = {};",
                    queries::DELETE_SIGN,
                    pq.0,
                    pq.1,
                    cmd.xyz.0,
                    cmd.xyz.1,
                    cmd.xyz.2,
                    cmd.face)
        } else {
            format!("{}({}, {}, {}, {}, {}, {}, \"{}\");",
                    queries::SET_SIGN,
                    pq.0,
                    pq.1,
                    cmd.xyz.0,
                    cmd.xyz.1,
                    cmd.xyz.2,
                    cmd.face,
                    cmd.sign.0)
        };

        self.conn.execute(query).unwrap();
    }
}

/// Return the chunk that a block falls in on one axis.
pub fn chunked(n: i32) -> i32 {
    (n as f32 / CHUNK_SIZE as f32).floor() as i32
}
