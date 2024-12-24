pub static USERS_SELECT: &str = "SELECT * FROM users";
pub static USERS_INSERT: &str = "INSERT INTO users (username, email, password, status, isadmin) VALUES ($1, $2, $3, true, false)";
pub static USERS_LOGIN: &str = "SELECT * FROM users WHERE username = $1";
pub static HARDWARES_SELECT: &str = "SELECT * FROM hardwares";
pub static HARDWARES_SELECT_ONE: &str = "SELECT * FROM hardwares WHERE id = $1";
pub static HARDWARES_INSERT: &str =
    "INSERT INTO hardwares (name, type, description) VALUES ($1, $2, $3)";
pub static HARDWARES_UPDATE: &str =
    "UPDATE hardwares SET name = $1, type = $2, description = $3 WHERE id = $4";
pub static HARDWARES_DELETE: &str = "DELETE FROM hardwares WHERE id = $1";
pub static NODES_SELECT: &str = "SELECT * FROM nodes";
pub static NODES_SELECT_ONE: &str = "SELECT * FROM nodes WHERE id = $1";
pub static FEEDS_SELECT_BY_NODE: &str = "SELECT * FROM feeds WHERE node_id = $1";
pub static FEEDS_INSERT: &str = "INSERT INTO feeds (node_id, time, value) VALUES ($1, $2, $3)";
