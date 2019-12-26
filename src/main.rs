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
#[cfg(test)]
mod test;
mod web;

fn main() {
    web::rocket().launch();
}
