use warp::Filter;

use mobc::{Connection, Pool};
use mobc_postgres::{tokio_postgres, PgConnectionManager};
use std::convert::Infallible;
use tokio_postgres::NoTls;

mod data;
mod db;
mod error;
mod handler;

/// A convenience type for the database connection.
type DbConn = Connection<PgConnectionManager<NoTls>>;
/// A convenience type for the database connection pool.
type DbPool = Pool<PgConnectionManager<NoTls>>;
/// A convenience type for fallible results.
type Result<T> = std::result::Result<T, warp::Rejection>;

#[tokio::main]
async fn main() {
    let db_pool = db::create_pool().expect("Database Connection Pool init failed.");
    db::init_db(&db_pool)
        .await
        .expect("Database Model init failed.");

    let health_route = warp::path!("health")
        .and(with_db(db_pool.clone()))
        .and_then(handler::health_handler);

    let todo = warp::path("todo");

    let todo_routes = todo
        .and(warp::get())
        .and(warp::query())
        .and(with_db(db_pool.clone()))
        .and_then(handler::list_todos_handler)
        .or(todo
            .and(warp::post())
            .and(warp::body::json())
            .and(with_db(db_pool.clone()))
            .and_then(handler::create_todo_handler))
        .or(todo
            .and(warp::put())
            .and(warp::path::param())
            .and(warp::body::json())
            .and(with_db(db_pool.clone()))
            .and_then(handler::update_todo_handler))
        .or(todo
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_db(db_pool.clone()))
            .and_then(handler::delete_todo_handler));

    let routes = health_route
        .or(todo_routes)
        .with(warp::cors().allow_any_origin())
        .recover(error::handle_rejection);

    println!(">>> Starting the server ...");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

/// A WARP Filter that for any route it extracts the `DbPool` and pass it along.<br/>
/// It can be added to a handler definition using the `.and()` method.
fn with_db(db_pool: DbPool) -> impl Filter<Extract = (DbPool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}
