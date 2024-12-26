#![allow(dead_code)]
// Messages
pub static MESSAGE_OK: &str = "ok";
pub static MESSAGE_SIGNUP_SUCCESS: &str = "Signup successfully";
pub static MESSAGE_LOGIN_SUCCESS: &str = "Login successfully";
pub static MESSAGE_LOGIN_FAILED: &str = "Wrong username or password, please try again";
pub static MESSAGE_USER_NOT_FOUND: &str = "User not found";
pub static MESSAGE_INTERNAL_SERVER_ERROR: &str = "Internal Server Error";
pub static MESSAGE_CHANGE_PASSWORD_SUCCESS: &str = "Change password successfully";

// Bad request messages
pub static MESSAGE_PASSWORD_NOT_MATCH: &str = "Old password does not match";
pub static MESSAGE_CHANGE_PASSWORD_FAILED: &str = "Change password failed";

// Conflict messages
pub static USERSNAME_ALREADY_EXIST: &str = "Username already exist";

// Auth messages
pub static MESSAGE_UNAUTHORIZED: &str = "Unauthorized";
pub static MESSAGE_TOKEN_EXPIRED: &str = "Token has expired";
pub static MESSAGE_INVALID_TOKEN: &str = "Invalid token";
pub static MESSAGE_UNAUTHENTICATED: &str = "Unauthenticated";

// Misc
pub static EMPTY: &str = "";
pub static MESSAGE_NOT_FOUND: &str = "Not found";
