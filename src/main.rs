#[macro_use]
extern crate diesel;
extern crate async_graphql;
extern crate dotenv;
extern crate serde;
#[macro_use]
extern crate serde_json;

use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_actors::ws;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::Schema;
use async_graphql_actix_web::{GQLRequest, GQLResponse, WSSubscription};

mod auth;
pub mod context;
pub mod db;
pub mod gql_schema;
pub mod models;
mod schema;

use auth::login;
use context::{CindyContext, CindyQueryContext};
use gql_schema::{CindySchema, MutationRoot, QueryRoot, SubscriptionRoot};

async fn index(
    schema: web::Data<CindySchema>,
    req: HttpRequest,
    gql_req: GQLRequest,
) -> GQLResponse {
    let headers = req.headers();
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().map(|v| v.to_owned()).ok());

    gql_req
        .into_inner()
        .data(CindyQueryContext::default().with_token(token))
        .execute(&schema)
        .await
        .into()
}

/*
async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql"),
        )))
}
*/

/*
async fn index_ws(
    schema: web::Data<BooksSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    ws::start_with_protocols(WSSubscription::new(&schema), &["graphql-ws"], &req, payload)
}
*/

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let ctx = CindyContext::default();
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
        /*
        .service(
            web::resource("/graphql")
                .guard(guard::Get())
                .guard(guard::Header("upgrade", "websocket"))
                .to(index_ws),
        )
        */
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
