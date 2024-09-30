use ntex::web;

mod users;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .configure(users::routes)
    );
}