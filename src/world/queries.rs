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
    sign (x, y, z, face); \
    CREATE TABLE IF NOT EXISTS light (\
    p INT NOT NULL, \
    q INT NOT NULL, \
    x INT NOT NULL, \
    y INT NOT NULL, \
    z INT NOT NULL, \
    w INT NOT NULL); \
    CREATE UNIQUE INDEX IF NOT EXISTS light_pqxyz_idx ON \
    light (p, q, x, y, z);"
;

/// Loads blocks from the database.
pub const LOAD_BLOCKS: &str = "SELECT p, q, x, y, z, w FROM block;";

/// Sets a block.
/* pub const SET_BLOCK: &str = "INSERT OR REPLACE INTO block (p, q, x, y, z, w) VALUES "; */
pub const SET_BLOCK: &str =
    "INSERT OR REPLACE INTO block (p, q, x, y, z, w) VALUES \
    (?, ?, ?, ?, ?, ?);";

/// Loads signs from the database.
pub const LOAD_SIGNS: &str = "SELECT p, q, x, y, z, face, text FROM sign;";

/// Sets a sign.
/* pub const SET_SIGN: &str = "INSERT OR REPLACE INTO sign (p, q, x, y, z, face, text) VALUES "; */
pub const SET_SIGN: &str =
    "INSERT OR REPLACE INTO sign (p, q, x, y, z, face, text) VALUES \
    (?, ?, ?, ?, ?, ?, ?);";

/// Deletes a sign on a specific face of a specific block.
pub const DELETE_INDIVIDUAL_SIGN: &str = "DELETE FROM sign WHERE x = ? and y = ? and z = ? and face = ?";

/// Deletes all signs on a block.
/* pub const DELETE_SIGN: &str = "DELETE FROM sign WHERE "; */
pub const DELETE_SIGNS: &str = "DELETE FROM sign WHERE x = ? AND y = ? AND z = ?";

/// Loads lights from the database.
pub const LOAD_LIGHTS: &str = "SELECT p, q, x, y, z, w FROM light;";

/// Sets a light.
pub const SET_LIGHT: &str =
    "INSERT OR REPLACE INTO light (p, q, x, y, z, w) VALUES \
    (?, ?, ?, ?, ?, ?);";

/* /// Commit the database transactions.
pub const COMMIT: &str = "COMMIT;";
*/
