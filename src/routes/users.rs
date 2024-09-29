use ntex::web;

use crate::controllers::users;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(users::get_all_users_controller));
    cfg.route("/{id}/", web::get().to(users::get_user_by_id_controller));
    cfg.route("/login/", web::post().to(users::login_controller));
    cfg.route("/signup/", web::post().to(users::signup_controller));
    cfg.route("/{id}/", web::put().to(users::change_password_controller));
}