use ntex::web;

mod hardwares;
mod users;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/users").configure(users::routes));
    cfg.service(web::scope("/hardwares").configure(hardwares::routes));
}
