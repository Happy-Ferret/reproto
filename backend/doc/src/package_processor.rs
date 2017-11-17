//! Processor for service declarations.

use backend::Environment;
use backend::errors::*;
use core::{Loc, RpDecl, RpVersionedPackage};
use doc_builder::DocBuilder;
use escape::Escape;
use macros::FormatAttribute;
use processor::Processor;

pub struct Data<'a> {
    pub package: &'a RpVersionedPackage,
    pub decls: &'a [&'a Loc<RpDecl>],
}

macro_rules! types_section {
    ($slf:ident, $var:ident, $name:expr) => {
        if !$var.is_empty() {
            html!($slf, h2 {class => "kind"} ~ $name);

            for v in $var {
                let url = $slf.type_url(&v.name)?;

                html!($slf, h3 {} => {
                    html!($slf, a {href => url} ~ v.name.parts.join("::"));
                });
            }
        }
    }
}

define_processor!(PackageProcessor, Data<'env>, self {
    use self::RpDecl::*;

    self.write_doc(|| {
        let mut types = Vec::new();
        let mut interfaces = Vec::new();
        let mut enums = Vec::new();
        let mut tuples = Vec::new();
        let mut services = Vec::new();

        for decl in self.body.decls {
            match *decl.value() {
                Type(ref ty) => types.push(ty),
                Interface(ref interface) => interfaces.push(interface),
                Enum(ref en) => enums.push(en),
                Tuple(ref tuple) => tuples.push(tuple),
                Service(ref service) => services.push(service),
            }
        }

        html!(self, section {class => "section-content"} => {
            html!(self, h1 {} => {
                html!(self, span {class => "kind"} ~ "package");
                html!(self, span {} ~ Escape(self.body.package.to_string().as_str()));
            });

            types_section!(self, types, "Types");
            types_section!(self, interfaces, "Interfaces");
            types_section!(self, enums, "Enums");
            types_section!(self, tuples, "Tuples");
            types_section!(self, services, "Services");
        });

        Ok(())
    })
});

impl<'env> PackageProcessor<'env> {}
