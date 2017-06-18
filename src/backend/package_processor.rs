use backend::errors::*;
use core::*;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use super::collecting::Collecting;
use super::environment::Environment;

pub trait PackageProcessor
    where Self: Sized
{
    type Out: Collecting<Processor = Self>;

    fn ext(&self) -> &str;

    fn env(&self) -> &Environment;

    fn package_prefix(&self) -> &Option<RpPackage>;

    fn out_path(&self) -> &Path;

    fn default_process(&self, out: &mut Self::Out, type_id: &RpTypeId, pos: &RpPos) -> Result<()>;

    fn process_interface(&self,
                         out: &mut Self::Out,
                         type_id: &RpTypeId,
                         pos: &RpPos,
                         _: Rc<RpInterfaceBody>)
                         -> Result<()> {
        self.default_process(out, type_id, pos)
    }

    fn process_type(&self,
                    out: &mut Self::Out,
                    type_id: &RpTypeId,
                    pos: &RpPos,
                    _: Rc<RpTypeBody>)
                    -> Result<()> {
        self.default_process(out, type_id, pos)
    }

    fn process_tuple(&self,
                     out: &mut Self::Out,
                     type_id: &RpTypeId,
                     pos: &RpPos,
                     _: Rc<RpTupleBody>)
                     -> Result<()> {
        self.default_process(out, type_id, pos)
    }

    fn process_enum(&self,
                    out: &mut Self::Out,
                    type_id: &RpTypeId,
                    pos: &RpPos,
                    _: Rc<RpEnumBody>)
                    -> Result<()> {
        self.default_process(out, type_id, pos)
    }

    fn process_service(&self,
                       out: &mut Self::Out,
                       type_id: &RpTypeId,
                       pos: &RpPos,
                       _: Rc<RpServiceBody>)
                       -> Result<()> {
        self.default_process(out, type_id, pos)
    }

    fn populate_files(&self) -> Result<BTreeMap<&RpVersionedPackage, Self::Out>> {
        self.do_populate_files(|_, _| Ok(()))
    }

    fn do_populate_files<'a, F>(&'a self,
                                mut callback: F)
                                -> Result<BTreeMap<&RpVersionedPackage, Self::Out>>
        where F: FnMut(&'a RpTypeId, &'a RpLoc<RpDecl>) -> Result<()>
    {
        let mut files = BTreeMap::new();

        // Process all types discovered so far.
        for (type_id, decl) in &self.env().decls {
            callback(type_id, decl)?;

            let mut out = files.entry(&type_id.package).or_insert_with(|| Self::Out::new());

            match ***decl {
                RpDecl::Interface(ref body) => {
                    self.process_interface(&mut out, type_id, decl.pos(), body.clone())?
                }
                RpDecl::Type(ref body) => {
                    self.process_type(&mut out, type_id, decl.pos(), body.clone())?
                }
                RpDecl::Tuple(ref body) => {
                    self.process_tuple(&mut out, type_id, decl.pos(), body.clone())?
                }
                RpDecl::Enum(ref body) => {
                    self.process_enum(&mut out, type_id, decl.pos(), body.clone())?
                }
                RpDecl::Service(ref body) => {
                    self.process_service(&mut out, type_id, decl.pos(), body.clone())?
                }
            };
        }

        Ok(files)
    }

    /// Identify if a character is unsafe for use in a package name.
    fn package_version_unsafe(c: char) -> bool {
        match c {
            '.' | '-' | '~' => true,
            _ => false,
        }
    }

    /// Default strategy for building the version package.
    fn version_package(input: &Version) -> String {
        format!("_{}", input).replace(Self::package_version_unsafe, "_")
    }

    /// Build the full package of a versioned package.
    ///
    /// This includes the prefixed configured in `self.options`, if specified.
    ///
    /// This uses a relatively safe strategy for encoding the version number. This can be adjusted
    /// by overriding `version_package`.
    fn package(&self, package: &RpVersionedPackage) -> RpPackage {
        self.package_prefix()
            .clone()
            .map(|prefix| prefix.join_versioned(package))
            .unwrap_or_else(|| package.clone())
            .into_package(Self::version_package)
    }

    fn resolve_full_path(&self, root_dir: &Path, package: &RpVersionedPackage) -> Result<PathBuf> {
        let mut full_path = root_dir.to_owned();
        let package = self.package(package);
        let mut iter = package.parts.iter().peekable();

        while let Some(part) = iter.next() {
            full_path = full_path.join(part);
        }

        // path to final file
        full_path.set_extension(self.ext());
        Ok(full_path)
    }

    fn setup_module_path(&self, root_dir: &Path, package: &RpVersionedPackage) -> Result<PathBuf> {
        let full_path = self.resolve_full_path(root_dir, package)?;

        if let Some(parent) = full_path.parent() {
            if !parent.is_dir() {
                debug!("+dir: {}", parent.display());
                fs::create_dir_all(parent)?;
            }
        }

        Ok(full_path)
    }

    fn write_files(&self, files: BTreeMap<&RpVersionedPackage, Self::Out>) -> Result<()> {
        let root_dir = &self.out_path();

        for (package, out) in files {
            let full_path = self.setup_module_path(root_dir, package)?;

            debug!("+module: {}", full_path.display());

            let mut f = File::create(full_path)?;
            let bytes = out.into_bytes(self)?;
            f.write_all(&bytes)?;
            f.flush()?;
        }

        Ok(())
    }
}
