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
    block (p, q, x, y, z);"
;

/// Loads blocks from the database.
pub const LOAD_BLOCKS: &str = "SELECT p, q, x, y, z, w FROM block";

/// The first part of the query for saving blocks.
/// Actual values must be appended.
pub const SET_BLOCK: &str = "INSERT OR REPLACE INTO block (p, q, x, y, z, w) VALUES ";

/*
// Needed for vanilla Craft, even though signs
// are not yet implemented for this server.
/// Removes signs from a block.
pub const REMOVE_SIGN: &str = "";

/// Commit the database transactions.
pub const COMMIT: &str = "COMMIT;";
*/
