use axum::{
    body::{boxed, Body, BoxBody},
    http::{Request, Response, StatusCode, Uri},
    response::AppendHeaders,
    routing::{get, post},
    Router,
};
use clap::Parser;
use config::Config;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use handlebars::Handlebars;
use lazy_static::lazy_static;
use static_init::dynamic;
use std::{net::SocketAddr, path::PathBuf};
use tower::ServiceExt;
use tower_http::services::ServeDir;

mod archer;
mod config;
mod db;
mod error;
mod models;
mod schema;

#[dynamic()]
pub static mut CONFIG: Config = Config::default();

#[dynamic()]
pub static mut HANDLEBARS: Handlebars<'static> = Handlebars::new();

#[derive(Parser, Debug)]
struct CliArgs {
    /// Path to config file
    #[arg(long, default_value_t = String::from("config.toml"))]
    config_file: String,

    /// Path to database file. Overwrites environment variable
    #[arg(long)]
    database_file: Option<String>,

    /// Path to email template
    #[arg(long, default_value_t = String::from("user_mail.tpl"))]
    mail_template_file: String,

    /// Path to email password file.
    /// Overwrites password from config
    #[arg(long)]
    mail_password_file: Option<PathBuf>,
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = CliArgs::parse();
    if let Some(db_file) = args.database_file {
        std::env::set_var("DATABASE_URL", db_file);
    }
    db::establish_connection()
        .run_pending_migrations(MIGRATIONS)
        .expect("Could not migrate database");

    *CONFIG.write() = {
        let mut config = load_config(&std::path::PathBuf::from(&args.config_file));
        if let Some(pswd) = args.mail_password_file {
            config.mail_server.smtp_password =
                std::fs::read_to_string(pswd).expect("Password file couldn't be read");
        }
        config
    };
    {
        let mut handlebars = HANDLEBARS.write();
        handlebars.set_strict_mode(true);
        handlebars.set_dev_mode(cfg!(debug_assertions));
        handlebars
            .register_template_file(
                "user_mail",
                std::path::PathBuf::from(args.mail_template_file),
            )
            .unwrap();
    }

    let api = Router::new()
        .route("/archers", post(archer::create_archer))
        .route("/archers", get(archer::list_archers));
    let app = Router::new()
        .nest_service(
            "/",
            get(handler).then(|res| async move {
                Ok((
                    AppendHeaders([(axum::http::header::CACHE_CONTROL, "no-cache")]),
                    res,
                ))
            }),
        )
        .nest_service("/api", api);

    let addr = SocketAddr::from(([0, 0, 0, 0], CONFIG.read().port));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let res = get_static_file(uri.clone()).await?;

    if res.status() == StatusCode::NOT_FOUND {
        // try with `.html`
        // TODO: handle if the Uri has query parameters
        match format!("{}.html", uri).parse() {
            Ok(uri_html) => get_static_file(uri_html).await,
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid URI".to_string())),
        }
    } else {
        Ok(res)
    }
}

async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    lazy_static! {
        static ref HTML_PATH: String =
            std::env::var("WEBPAGE").unwrap_or_else(|_| "../frontend/dist".to_string());
    }

    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    match ServeDir::new(&*HTML_PATH).oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}

fn load_config(path: &std::path::Path) -> Config {
    let toml_config = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Couldn't read file from path {:?}", path));
    toml::from_str(&toml_config).expect("Couldn't parse config content!")
}
