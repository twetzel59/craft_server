// These queries come from Michael Fogleman's server.
// See the code here: https://github.com/fogleman/Craft/blob/master/server.py

pub const INITIAL: [&str; 2] = [
    "CREATE TABLE IF NOT EXISTS block (\
    p INT NOT NULL, \
    q INT NOT NULL, \
    x INT NOT NULL, \
    y INT NOT NULL, \
    z INT NOT NULL, \
    w INT NOT NULL);",
    "CREATE UNIQUE INDEX IF NOT EXISTS block_pqxyz_idx ON \
    block (p, q, x, y, z);"
];
