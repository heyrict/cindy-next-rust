#[macro_use]
extern crate diesel;
extern crate async_graphql;
extern crate dotenv;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_actors::ws;
use async_graphql::Schema;
use async_graphql_actix_web::{Request, Response, WSSubscription};

mod auth;
pub mod context;
pub mod db;
pub mod gql_schema;
pub mod models;
mod schema;
mod broker;

use auth::{login, signup};
use context::{GlobalCtx, RequestCtx};
use gql_schema::{CindySchema, MutationRoot, QueryRoot, SubscriptionRoot};

lazy_static! {
    pub static ref ADMIN_SECRET: String =
        dotenv::var("ADMIN_SECRET").expect("Invalid ADMIN_SECRET env var");
}

async fn index(schema: web::Data<CindySchema>, req: HttpRequest, gql_req: Request) -> Response {
    let headers = req.headers();
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().map(|v| v.to_owned()).ok());
    let admin_secret = headers
        .get("X-CINDY-ADMIN-SECRET")
        .and_then(|value| value.to_str().map(|v| v.to_owned()).ok());

    schema
        .execute(
            gql_req.into_inner().data(
                RequestCtx::default()
                    .with_token(token)
                    .with_secret(admin_secret),
            ),
        )
        .await
        .into()
}

async fn index_ws(
    schema: web::Data<CindySchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    ws::start_with_protocols(
        WSSubscription::new(Schema::clone(&*schema)),
        &["graphql-ws"],
        &req,
        payload,
    )
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let ctx = GlobalCtx::default();
    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(ctx.clone())
        .finish();

    println!("Endpoint: http://localhost:8000/graphql");

    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .data(ctx.clone())
            .service(web::resource("/graphql").guard(guard::Post()).to(index))
            .service(web::resource("/login").guard(guard::Post()).to(login))
            .service(web::resource("/signup").guard(guard::Post()).to(signup))
            .service(
                web::resource("/graphql")
                    .guard(guard::Get())
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(index_ws),
            )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
