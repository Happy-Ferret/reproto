//! Compiler for generating documentation.

use super::{DOC_CSS_NAME, NORMALIZE_CSS_NAME};
use backend::errors::*;
use core::{Loc, RpDecl, RpVersionedPackage};
use doc_backend::DocBackend;
use doc_builder::DocBuilder;
use enum_processor::EnumProcessor;
use genco::IoFmt;
use index_processor::{Data as IndexData, IndexProcessor};
use interface_processor::InterfaceProcessor;
use package_processor::{Data as PackageData, PackageProcessor};
use processor::Processor;
use service_processor::ServiceProcessor;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tuple_processor::TupleProcessor;
use type_processor::TypeProcessor;

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
        let mut by_package: BTreeMap<&RpVersionedPackage, Vec<&Loc<RpDecl>>> = BTreeMap::new();

        self.backend.env.for_each_decl(|decl| {
            let package = decl.name().package.clone().into_package(|v| v.to_string());

            by_package
                .entry(&decl.name().package)
                .or_insert_with(Vec::new)
                .push(&decl);

            // maintain to know where to import static resources from.
            let mut root = Vec::new();
            let mut path = self.out_path.to_owned();

            for part in package.parts {
                root.push("..");
                path = path.join(part.as_str());

                if !path.is_dir() {
                    debug!("+dir: {}", path.display());
                    fs::create_dir_all(&path)?;
                }
            }

            let name = decl.name().parts.join(".");

            // complete path to root and static resources
            let root = root.join("/");

            let mut buffer = String::new();

            match *decl.value() {
                Interface(ref body) => {
                    InterfaceProcessor {
                        out: RefCell::new(DocBuilder::new(&mut buffer)),
                        env: &self.backend.env,
                        root: &root,
                        body: body,
                    }.process()?;
                }
                Type(ref body) => {
                    TypeProcessor {
                        out: RefCell::new(DocBuilder::new(&mut buffer)),
                        env: &self.backend.env,
                        root: &root,
                        body: body,
                    }.process()?;
                }
                Tuple(ref body) => {
                    TupleProcessor {
                        out: RefCell::new(DocBuilder::new(&mut buffer)),
                        env: &self.backend.env,
                        root: &root,
                        body: body,
                    }.process()?;
                }
                Enum(ref body) => {
                    EnumProcessor {
                        out: RefCell::new(DocBuilder::new(&mut buffer)),
                        env: &self.backend.env,
                        root: &root,
                        body: body,
                    }.process()?;
                }
                Service(ref body) => {
                    ServiceProcessor {
                        out: RefCell::new(DocBuilder::new(&mut buffer)),
                        env: &self.backend.env,
                        root: &root,
                        body: body,
                    }.process()?;
                }
            }

            let out = path.join(format!("{}.{}.html", decl.kind(), name));
            let mut f = File::create(&out)?;
            f.write_all(buffer.as_bytes())?;
            debug!("+file: {}", out.display());

            Ok(())
        })?;

        self.write_index(&by_package)?;

        for (package, decls) in by_package.iter() {
            self.write_package(*package, decls)?;
        }

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

    /// Write the package index file index file.
    fn write_package(&self, package: &RpVersionedPackage, decls: &[&Loc<RpDecl>]) -> Result<()> {
        let mut path = self.out_path.to_owned();

        let mut root = Vec::new();

        for part in package.into_package(|v| v.to_string()).parts {
            root.push("..");
            path = path.join(part);
        }

        let index_html = path.join("index.html");
        let mut f = File::create(&index_html)?;

        PackageProcessor {
            out: RefCell::new(DocBuilder::new(&mut IoFmt(&mut f))),
            env: &self.backend.env,
            root: &root.join("/"),
            body: &PackageData {
                package: package,
                decls: decls,
            },
        }.process()?;

        debug!("+file: {}", index_html.display());
        Ok(())
    }

    /// Write the root index file.
    fn write_index(
        &self,
        by_package: &BTreeMap<&RpVersionedPackage, Vec<&Loc<RpDecl>>>,
    ) -> Result<()> {
        let index_html = self.out_path.join("index.html");
        let mut f = File::create(&index_html)?;

        let packages = by_package.keys().map(|p| *p).collect();

        IndexProcessor {
            out: RefCell::new(DocBuilder::new(&mut IoFmt(&mut f))),
            env: &self.backend.env,
            root: &".",
            body: &IndexData { packages: packages },
        }.process()?;

        debug!("+file: {}", index_html.display());
        Ok(())
    }
}
