//! Compiler for generating documentation.

use super::{DOC_CSS_NAME, NORMALIZE_CSS_NAME};
use backend::errors::*;
use core::{Loc, RpDecl, RpVersionedPackage};
use doc_backend::DocBackend;
use doc_builder::DefaultDocBuilder;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const NORMALIZE_CSS: &[u8] = include_bytes!("static/normalize.css");

pub struct DocCompiler<'a> {
    pub backend: &'a DocBackend,
    pub out_path: PathBuf,
    pub skip_static: bool,
}

impl<'a> DocCompiler<'a> {
    pub fn new(backend: &'a DocBackend, out_path: PathBuf, skip_static: bool) -> DocCompiler {
        DocCompiler {
            backend: backend,
            out_path: out_path,
            skip_static: skip_static,
        }
    }

    pub fn compile(&self) -> Result<()> {
        use self::RpDecl::*;

        // index by package
        let mut by_package: HashMap<&RpVersionedPackage, Vec<&Loc<RpDecl>>> = HashMap::new();

        self.backend.env.for_each_toplevel_decl(|decl| {
            let package = &decl.name().package;

            by_package
                .entry(&decl.name().package)
                .or_insert_with(Vec::new)
                .push(&decl);

            // maintain to know where to import static resources from.
            let mut root = Vec::new();
            let mut path = self.out_path.to_owned();

            for part in package.clone().into_package(|v| v.to_string()).parts {
                root.push("..");
                path = path.join(part);

                if !path.is_dir() {
                    debug!("+dir: {}", path.display());
                    fs::create_dir_all(&path)?;
                }
            }

            let name = decl.name().parts.join(".");

            // complete path to root and static resources
            let root = root.join("/");

            let mut buffer = String::new();

            self.backend.write_doc(
                &mut DefaultDocBuilder::new(&mut buffer),
                root.as_str(),
                |out| match *decl.value() {
                    Interface(ref b) => self.backend.process_interface(out, b),
                    Type(ref b) => self.backend.process_type(out, b),
                    Tuple(ref b) => self.backend.process_tuple(out, b),
                    Enum(ref b) => self.backend.process_enum(out, b),
                    Service(ref b) => self.backend.process_service(out, b),
                },
            )?;

            let out = path.join(format!("{}.html", name));
            let mut f = File::create(&out)?;
            f.write_all(buffer.as_bytes())?;
            debug!("+file: {}", out.display());

            Ok(())
        })?;

        if !self.skip_static {
            self.write_stylesheets()?;
        }

        Ok(())
    }

    fn write_stylesheets(&self) -> Result<()> {
        if !self.out_path.is_dir() {
            debug!("+dir: {}", self.out_path.display());
            fs::create_dir_all(&self.out_path)?;
        }

        let normalize_css = self.out_path.join(NORMALIZE_CSS_NAME);

        debug!("+css: {}", normalize_css.display());
        let mut f = fs::File::create(normalize_css)?;
        f.write_all(NORMALIZE_CSS)?;

        let doc_css = self.out_path.join(DOC_CSS_NAME);

        let content = self.backend.themes.get(self.backend.theme.as_str());

        if let Some(content) = content {
            debug!("+css: {}", doc_css.display());
            let mut f = fs::File::create(doc_css)?;
            f.write_all(content)?;
        } else {
            return Err(format!("no such theme: {}", &self.backend.theme).into());
        }

        Ok(())
    }
}
