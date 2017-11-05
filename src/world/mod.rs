//! This module contains the necessary functionality for representing
//! the world, both on disk and in memory.

mod queries;

const FILE: &str = "world.db";

use std::collections::hash_map;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use sqlite::{self, Connection, Statement};

/// The square X and Z dimensions of a world sector.
pub const CHUNK_SIZE: u8 = 32;

/// Type of block IDs.
#[derive(Clone, Debug)]
pub struct Block(pub i8);

impl Block {
    /// Returns wheather the block is empty air.
    pub fn is_air(&self) -> bool {
        self.0 == 0
    }
}

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

struct DatabaseThread<'l> {
    _conn: &'l Connection,
    statements: PreparedStatements<'l>,
    rx: mpsc::Receiver<DatabaseCommand>,
}

impl<'l> DatabaseThread<'l> {
    fn run(conn: Connection, rx: mpsc::Receiver<DatabaseCommand>) {
        thread::spawn(move || {
            let mut d = DatabaseThread {
                _conn: &conn,
                statements: PreparedStatements::new(&conn),
                rx,
            };

            d.database_thread();
        });
    }

    fn database_thread(&mut self) {
        use std::time::Duration;

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
    }

    fn handle_set_block(&mut self, cmd: &SetBlockCommand) {
        {
            let s = self.statements.set_block();

            s.0.bind(1, cmd.pq.0 as i64).unwrap();
            s.0.bind(2, cmd.pq.1 as i64).unwrap();
            s.0.bind(3, cmd.xyz.0 as i64).unwrap();
            s.0.bind(4, cmd.xyz.1 as i64).unwrap();
            s.0.bind(5, cmd.xyz.2 as i64).unwrap();
            s.0.bind(6, cmd.block.0 as i64).unwrap();
        }

        if cmd.block.is_air() {
            let s = self.statements.delete_signs();

            s.0.bind(1, cmd.xyz.0 as i64).unwrap();
            s.0.bind(2, cmd.xyz.1 as i64).unwrap();
            s.0.bind(3, cmd.xyz.2 as i64).unwrap();
        }
    }

    fn handle_set_sign(&mut self, cmd: &SetSignCommand) {
        if cmd.sign.0 == "" {
            let s = self.statements.delete_individual_sign();

            s.0.bind(1, cmd.xyz.0 as i64).unwrap();
            s.0.bind(2, cmd.xyz.1 as i64).unwrap();
            s.0.bind(3, cmd.xyz.2 as i64).unwrap();
            s.0.bind(4, cmd.face as i64).unwrap();
        } else {
            use ::std::ops::Deref;

            let pq = (chunked(cmd.xyz.0), chunked(cmd.xyz.2));
            let s = self.statements.set_sign();

            s.0.bind(1, pq.0 as i64).unwrap();
            s.0.bind(2, pq.1 as i64).unwrap();
            s.0.bind(3, cmd.xyz.0 as i64).unwrap();
            s.0.bind(4, cmd.xyz.1 as i64).unwrap();
            s.0.bind(5, cmd.xyz.2 as i64).unwrap();
            s.0.bind(6, cmd.face as i64).unwrap();
            s.0.bind(7, cmd.sign.0.deref()).unwrap();
        }
    }
}

/// Return the chunk that a block falls in on one axis.
pub fn chunked(n: i32) -> i32 {
    (n as f32 / CHUNK_SIZE as f32).floor() as i32
}

struct PreparedStatements<'l> {
    set_block: Statement<'l>,
    set_sign: Statement<'l>,
    delete_individual_sign: Statement<'l>,
    delete_signs: Statement<'l>,
}

impl<'l> PreparedStatements<'l> {
    fn new(conn: &Connection) -> PreparedStatements {
        PreparedStatements {
            set_block: conn.prepare(queries::SET_BLOCK).unwrap(),
            set_sign: conn.prepare(queries::SET_SIGN).unwrap(),
            delete_individual_sign: conn.prepare(queries::DELETE_INDIVIDUAL_SIGN).unwrap(),
            delete_signs: conn.prepare(queries::DELETE_SIGNS).unwrap(),
        }
    }

    fn set_block<'p>(&'p mut self) -> StatementWrapper<'l, 'p> {
        StatementWrapper(&mut self.set_block)
    }

    fn set_sign<'p>(&'p mut self) -> StatementWrapper<'l, 'p> {
        StatementWrapper(&mut self.set_sign)
    }

    fn delete_individual_sign<'p>(&'p mut self) -> StatementWrapper<'l, 'p> {
        StatementWrapper(&mut self.delete_individual_sign)
    }

    fn delete_signs<'p>(&'p mut self) -> StatementWrapper<'l, 'p> {
        StatementWrapper(&mut self.delete_signs)
    }
}

struct StatementWrapper<'l, 'p>(&'p mut Statement<'l>) where 'l: 'p;

impl<'l, 'p> Drop for StatementWrapper<'l, 'p> {
    fn drop(&mut self) {
        use ::sqlite::State;

        while let Ok(State::Row) = self.0.next() {}

        let _ = self.0.reset();
    }
}
