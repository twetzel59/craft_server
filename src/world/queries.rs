// These queries come from Michael Fogleman's server.
// See the code here: https://github.com/fogleman/Craft/blob/master/server.py

/// Sets up the database.
pub const INITIAL: &str =
    "CREATE TABLE IF NOT EXISTS block (\
    p INT NOT NULL, \
    q INT NOT NULL, \
    x INT NOT NULL, \
    y INT NOT NULL, \
    z INT NOT NULL, \
    w INT NOT NULL); \
    CREATE UNIQUE INDEX IF NOT EXISTS block_pqxyz_idx ON \
    block (p, q, x, y, z); \
    CREATE TABLE IF NOT EXISTS sign (\
    p INT NOT NULL, \
    q INT NOT NULL, \
    x INT NOT NULL, \
    y INT NOT NULL, \
    z INT NOT NULL, \
    face INT NOT NULL, \
    text TEXT NOT NULL); \
    CREATE INDEX IF NOT EXISTS sign_pq_idx ON \
    sign (p, q); \
    CREATE UNIQUE INDEX IF NOT EXISTS sign_xyzface_idx ON \
    sign (x, y, z, face);"
;

/// Loads blocks from the database.
pub const LOAD_BLOCKS: &str = "SELECT p, q, x, y, z, w FROM block";

/// The first part of the query for saving blocks.
/// Actual values must be appended.
pub const SET_BLOCK: &str = "INSERT OR REPLACE INTO block (p, q, x, y, z, w) VALUES ";

/// Loads signs from the database.
pub const LOAD_SIGNS: &str = "SELECT p, q, x, y, z, face, text FROM sign";

/// The first part of the query for storing signs.
/// Actual values must be appended.
pub const SET_SIGN: &str = "INSERT OR REPLACE INTO sign (p, q, x, y, z, face, text) VALUES ";

/// The first part of the query for deleting signs.
/// Actual values must be appended.
pub const DELETE_SIGN: &str = "DELETE FROM sign WHERE ";

/*
// Needed for vanilla Craft, even though signs
// are not yet implemented for this server.
/// Removes signs from a block.
pub const REMOVE_SIGN: &str = "";

/// Commit the database transactions.
pub const COMMIT: &str = "COMMIT;";
*/
