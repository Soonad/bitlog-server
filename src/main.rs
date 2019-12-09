#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

mod db;
mod web;
mod macros;
mod model;

fn main() {
    web::run();
}