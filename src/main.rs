#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate rocket_okapi;

mod db;
mod macros;
mod model;
mod web;

fn main() {
    let cors_options: rocket_cors::CorsOptions = rocket_cors::CorsOptions::default();
    let cors = cors_options.to_cors().expect("Cors");

    rocket::ignite()
        .mount(
            "/",
            routes_with_openapi![web::get_messages, web::create_message],
        )
        .attach(db::RocketConn::fairing())
        .attach(cors)
        .launch();
}
