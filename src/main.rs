use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod auth_middleware;
mod routes;
use routes::auth::{login, me, register};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    println!("Connected to database!");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(register)
            .service(login)
            .service(me)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
