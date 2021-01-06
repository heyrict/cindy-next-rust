#[macro_use]
extern crate diesel;
extern crate async_graphql;
extern crate dotenv;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use actix_cors::Cors;
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use async_graphql::Schema;
use async_graphql_actix_web::{Request, Response, WSSubscription};
use std::convert::TryInto;
use std::io::Write;
use time::Duration;

#[macro_use]
pub mod models;

mod auth;
mod broker;
pub mod context;
pub mod db;
pub mod gql_schema;
mod schema;

use auth::{login, signup, Role};
use context::{GlobalCtx, RequestCtx};
use gql_schema::{CindySchema, MutationRoot, QueryRoot, SubscriptionRoot};

lazy_static! {
    pub static ref ADMIN_SECRET: String =
        dotenv::var("ADMIN_SECRET").expect("Invalid ADMIN_SECRET env var");
}

async fn index(schema: web::Data<CindySchema>, req: HttpRequest, gql_req: Request) -> Response {
    const DEFAULT_OP_NAME: &str = "_";

    let headers = req.headers();
    let connection_info = req.connection_info();

    // Authorization info
    let token = headers.get("Authorization").and_then(|value| {
        value
            .to_str()
            .ok()
            // Drop `Bearer `
            .and_then(|v| v.splitn(2, ' ').nth(1))
            .map(|v| v.to_string())
    });
    let admin_secret = headers
        .get("X-CINDY-ADMIN-SECRET")
        .and_then(|value| value.to_str().map(|v| v.to_owned()).ok());
    let ctx = RequestCtx::default()
        .with_token(token)
        .with_secret(admin_secret);

    // Logging the IP address
    let gql_req = gql_req.into_inner();
    let op_name = gql_req
        .operation_name
        .clone()
        .unwrap_or(DEFAULT_OP_NAME.to_string());
    let ip_addr = if let Some(header_real_ip) = dotenv::var("HEADER_REAL_IP").ok() {
        headers
            .get(header_real_ip)
            .and_then(|ip| ip.to_str().ok())
            .or_else(|| connection_info.remote_addr())
    } else {
        connection_info.remote_addr()
    };
    let user = match ctx.get_role() {
        Role::Admin => {
            if let Some(user) = ctx.get_user() {
                format!("Admin<{}:{}>", &user.id, &user.nickname)
            } else {
                "Admin".to_string()
            }
        }
        Role::Guest => "Guest".to_string(),
        Role::User => {
            if let Some(user) = ctx.get_user() {
                format!("User<{}:{}>", &user.id, &user.nickname)
            } else {
                "User<?>".to_string()
            }
        }
    };
    info!(
        "({}) /graphql: {}: {}({})",
        ip_addr.unwrap_or_default(),
        user,
        op_name,
        &gql_req.variables
    );

    schema.execute(gql_req.data(ctx)).await.into()
}

async fn index_ws(
    schema: web::Data<CindySchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    WSSubscription::start(Schema::clone(&*schema), &req, payload)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Spawn cache cleaner
    tokio::spawn(async move {
        loop {
            tokio::time::delay_for(
                Duration::day()
                    .try_into()
                    .expect("Error converting a day to std::Duration"),
            )
            .await;
            debug!("Cleaning up cache");
            broker::cleanup();
        }
    });

    // Setup logger
    dotenv::dotenv().expect("Unable to setup dotenv");
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {} - {}",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
                record.level(),
                record.module_path().unwrap_or_default(),
                record.args()
            )
        })
        .init();

    let endpoint = dotenv::var("ENDPOINT").unwrap_or("127.0.0.1:8000".to_string());
    let ctx = GlobalCtx::default();
    let schema = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot::default(),
    )
    .data(ctx.clone())
    .finish();

    info!("Server started on: http://{}/graphql", &endpoint);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                let allowed_origins = dotenv::var("ALLOWED_ORIGINS").unwrap_or(String::new());
                let mut allowed_origins = allowed_origins.split(",");

                let origin: &str = if let Ok(origin) = std::str::from_utf8(origin.as_bytes()) {
                    origin
                } else {
                    return false;
                };

                allowed_origins
                    .find(|allowed| {
                        if allowed.len() == 0 {
                            return false;
                        };
                        let bytes = allowed.as_bytes();
                        let head_match = bytes[0] == b'*';
                        let tail_match = bytes[allowed.len() - 1] == b'*';

                        if head_match && tail_match {
                            if let Some(contents) = allowed.get(1..allowed.len() - 1) {
                                origin.contains(contents)
                            } else {
                                false
                            }
                        } else if head_match {
                            if let Some(contents) = allowed.get(1..) {
                                origin.ends_with(contents)
                            } else {
                                false
                            }
                        } else if tail_match {
                            if let Some(contents) = allowed.get(1..) {
                                origin.starts_with(contents)
                            } else {
                                false
                            }
                        } else {
                            allowed == &origin
                        }
                    })
                    .is_some()
            })
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
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
    .bind(&endpoint)?
    .run()
    .await
}
