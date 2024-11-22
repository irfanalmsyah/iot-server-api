use crate::handlers::hardwares;
use ntex::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(hardwares::get_all_hardwares_controller));
}
