#![recursion_limit = "1000"]

extern crate clap;
extern crate linked_hash_map;
extern crate pulldown_cmark;
extern crate serde_json;
extern crate reproto_core;
extern crate reproto_parser;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate codeviz;

pub mod backend;
pub mod commands;
pub mod errors;
pub mod logger;
pub mod naming;
pub mod options;

// external parts

pub(crate) use reproto_core as core;
pub(crate) use reproto_parser as parser;
