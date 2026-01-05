use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod auth_middleware;
mod db;
mod errors;
mod models;
mod routes;
mod services;

use routes::auth::{login, me, register};
use routes::multisig::{create_multisig, get_multisig, list_multisigs};
use routes::proposal::{
    activate_proposal, approve_proposal, create_proposal, execute_proposal, get_proposal,
    get_proposal_approvals, list_proposals, reject_proposal,
};

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
            .service(
                web::scope("/auth")
                    .service(register)
                    .service(login)
                    .service(me),
            )
            .service(
                web::scope("/multisigs")
                    .service(create_multisig)
                    .service(list_multisigs)
                    .service(get_multisig)
                    .service(
                        web::scope("/{multisig_id}/proposals")
                            .service(create_proposal)
                            .service(list_proposals),
                    ),
            )
            .service(
                web::scope("/proposals")
                    .service(get_proposal)
                    .service(activate_proposal)
                    .service(approve_proposal)
                    .service(execute_proposal)
                    .service(reject_proposal)
                    .service(get_proposal_approvals),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
