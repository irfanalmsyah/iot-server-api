use ntex::web;

pub async fn get_hardwares() -> impl web::Responder {
    web::HttpResponse::Ok().body("list of hardwares")
}
