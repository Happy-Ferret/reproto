//! Chrono module for Rust.
#![allow(unused)]

use backend::errors::*;
use genco::Rust;
use listeners::Listeners;
use rust_options::RustOptions;

pub struct Module {}

impl Module {
    pub fn new() -> Module {
        Module {}
    }
}

impl Listeners for Module {}
