//! Processor for service declarations.

use backend::Environment;
use backend::errors::*;
use core::RpVersionedPackage;
use doc_builder::DocBuilder;
use escape::Escape;
use macros::FormatAttribute;
use processor::Processor;

pub struct Data<'a> {
    pub packages: Vec<&'a RpVersionedPackage>,
}

define_processor!(IndexProcessor, Data<'env>, self {
    self.write_doc(|| {
        html!(self, section {class => "section-content"} => {
            html!(self, h1 {} ~ "index");

            for package in &self.body.packages {
                let url = package.into_package(|v| v.to_string()).parts.join("/");

                html!(self, h2 {} => {
                    html!(self, span {class => "kind"} ~ "package");
                    html!(self, a {href => url} ~ Escape(package.to_string().as_str()));
                });
            }
        });

        Ok(())
    })
});

impl<'env> IndexProcessor<'env> {}
