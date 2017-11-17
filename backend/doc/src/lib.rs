#![recursion_limit = "1000"]
#[macro_use]
extern crate log;
extern crate genco;
extern crate reproto_backend as backend;
extern crate reproto_core as core;
extern crate pulldown_cmark;

#[macro_use]
mod macros;
mod doc_backend;
mod doc_builder;
mod doc_compiler;
mod doc_listeners;
mod doc_options;
mod escape;
mod processor;
mod service_processor;
mod tuple_processor;
mod type_processor;
mod enum_processor;
mod interface_processor;
mod index_processor;
mod package_processor;

pub const NORMALIZE_CSS_NAME: &str = "normalize.css";
pub const DOC_CSS_NAME: &str = "doc.css";
pub const EXT: &str = "html";
pub const INDEX: &str = "index";
pub const DEFAULT_THEME: &str = "light";

use self::backend::{App, Arg, ArgMatches, CompilerOptions, Environment, Options};
use self::backend::errors::*;
use self::doc_backend::DocBackend;
use self::doc_compiler::DocCompiler;
use self::doc_listeners::DocListeners;
use self::doc_options::DocOptions;

fn setup_module(module: &str) -> Result<Box<DocListeners>> {
    let _module: Box<DocListeners> = match module {
        _ => return Err(format!("No such module: {}", module).into()),
    };
}

pub fn setup_listeners(options: Options) -> Result<(DocOptions, Box<DocListeners>)> {
    let mut listeners: Vec<Box<DocListeners>> = Vec::new();

    for module in &options.modules {
        listeners.push(setup_module(module)?);
    }

    let mut options = DocOptions::new();

    for listener in &listeners {
        listener.configure(&mut options)?;
    }

    Ok((options, Box::new(listeners)))
}

pub fn shared_options<'a, 'b>(out: App<'a, 'b>) -> App<'a, 'b> {
    let out = out.arg(
        Arg::with_name("theme")
            .long("theme")
            .takes_value(true)
            .help("Theme to use"),
    );

    let out = out.arg(Arg::with_name("skip_static").long("skip-static").help(
        "Skip building \
         with static \
         files",
    ));

    out
}

pub fn compile_options<'a, 'b>(out: App<'a, 'b>) -> App<'a, 'b> {
    shared_options(out).about("Compile Documentation")
}

pub fn verify_options<'a, 'b>(out: App<'a, 'b>) -> App<'a, 'b> {
    shared_options(out).about("Verify for Documentation")
}

pub fn compile(
    env: Environment,
    options: Options,
    compiler_options: CompilerOptions,
    matches: &ArgMatches,
) -> Result<()> {
    let theme = matches
        .value_of("theme")
        .unwrap_or(DEFAULT_THEME)
        .to_owned();
    let skip_static = matches.is_present("skip_static");

    let (options, listeners) = setup_listeners(options)?;
    let backend = DocBackend::new(env, options, listeners, theme);
    let compiler = DocCompiler::new(&backend, compiler_options.out_path, skip_static);
    compiler.compile()
}

pub fn verify(env: Environment, options: Options, _matches: &ArgMatches) -> Result<()> {
    let theme = String::from("light");
    let (options, listeners) = setup_listeners(options)?;
    let backend = DocBackend::new(env, options, listeners, theme);
    backend.verify()
}
