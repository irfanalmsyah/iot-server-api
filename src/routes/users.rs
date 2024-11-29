use ntex::web;

use crate::controllers::users;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(users::get_all_users_controller));
}