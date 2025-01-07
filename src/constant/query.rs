pub static USERS_SELECT: &str = "SELECT id, username, email, status, isadmin FROM users";
pub static USERS_INSERT: &str = "INSERT INTO users (username, email, password, status, isadmin) VALUES ($1, $2, $3, false, false)";
pub static USERS_SELECT_BY_USERNAME: &str = "SELECT * FROM users WHERE username = $1";
pub static USERS_SELECT_BY_ID: &str =
    "SELECT id, username, email, status, isadmin FROM users WHERE id = $1;";
pub static USERS_SELECT_BY_USERNAME_AND_EMAIL: &str =
    "SELECT * FROM users WHERE username = $1 AND email = $2";
pub static USERS_UPDATE_STATUS_BY_USERNAME: &str =
    "UPDATE users SET status = true WHERE username = $1";
pub static USERS_UPDATE_PASSWORD_BY_USERNAME: &str =
    "UPDATE users SET password = $1 WHERE username = $2";
pub static HARDWARES_SELECT: &str = "SELECT * FROM hardwares";
pub static HARDWARES_SELECT_BY_ID: &str = "SELECT * FROM hardwares WHERE id = $1";
pub static HARDWARES_INSERT: &str =
    "INSERT INTO hardwares (name, type, description) VALUES ($1, $2, $3)";
pub static HARDWARES_UPDATE_BY_ID: &str =
    "UPDATE hardwares SET name = $1, type = $2, description = $3 WHERE id = $4";
pub static HARDWARES_DELETE_BY_ID: &str = "DELETE FROM hardwares WHERE id = $1";
pub static NODES_SELECT: &str = "SELECT * FROM nodes";
pub static NODES_SELECT_BY_USER_OR_ISPUBLIC: &str =
    "SELECT * FROM nodes WHERE user_id = $1 or ispublic = true";
pub static NODES_SELECT_BY_ID: &str = "SELECT * FROM nodes WHERE id = $1";
pub static NODES_SELECT_BY_ID_AND_BY_USER_OR_ISPUBLIC: &str =
    "SELECT * FROM nodes WHERE id = $1 AND (user_id = $2 OR ispublic = true)";
pub static NODES_INSERT: &str = "INSERT INTO nodes (user_id, hardware_id, name, location, hardware_sensor_ids, hardware_sensor_names, ispublic) VALUES ($1, $2, $3, $4, $5, $6, $7)";
pub static NODES_UPDATE_BY_ID: &str = "UPDATE nodes SET hardware_id = $1, name = $2, location = $3, hardware_sensor_ids = $4, hardware_sensor_names = $5, ispublic = $6 WHERE id = $7";
pub static NODES_UPDATE_BY_ID_AND_USER_ID: &str = "UPDATE nodes SET hardware_id = $1, name = $2, location = $3, hardware_sensor_ids = $4, hardware_sensor_names = $5, ispublic = $6 WHERE id = $7 AND user_id = $8";
pub static NODES_DELETE_BY_ID: &str = "DELETE FROM nodes WHERE id = $1";
pub static NODES_DELETE_BY_ID_AND_USER_ID: &str =
    "DELETE FROM nodes WHERE id = $1 AND user_id = $2";
pub static FEEDS_SELECT_BY_NODE_ID: &str = "SELECT * FROM feeds WHERE node_id = $1";
pub static FEEDS_INSERT: &str = "INSERT INTO feeds (node_id, time, value) SELECT $1, $2, $3 FROM nodes WHERE id = $1 AND user_id = $4;";
pub static FEEDS_INSERT_MQTT: &str = "INSERT INTO feeds (node_id, time, value) VALUES ($1, $2, $3)";
pub static HARDWARES_VALIDATE_SENSOR_IDS: &str = "SELECT COUNT(*) FROM unnest($1::INTEGER[]) AS sensor_id WHERE NOT EXISTS (SELECT 1 FROM hardwares WHERE id = sensor_id AND type = 'sensor')";
