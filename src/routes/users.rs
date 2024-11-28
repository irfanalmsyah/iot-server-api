use ntex::web;

use crate::handlers::users::{
    get_all_users_controller, get_user_by_id_controller, login_controller, signup_controller,
};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::get().to(get_all_users_controller)),
    );
    cfg.route("/{id}/", web::get().to(get_user_by_id_controller));
    cfg.route("/login/", web::post().to(login_controller));
    cfg.route("/signup/", web::post().to(signup_controller));
    // cfg.route("/{id}/", web::put().to(users::change_password_controller));
}
