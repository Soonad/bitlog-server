#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

mod db;
mod web;
mod macros;
mod model;

fn main() {
  rocket::ignite()
    .mount("/", routes![web::get_messages, web::create_message])
    .attach(db::RocketConn::fairing())
    .launch();
}