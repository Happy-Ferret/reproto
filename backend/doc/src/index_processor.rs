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

define_processor!(IndexProcessor, Data<'env>, self,
    process => {
        self.write_doc(|| {
            html!(self, section {class => "section-content"} => {
                html!(self, h1 {class => "section-title"} ~ "Index");

                html!(self, h2 {class => "kind"} ~ "Packages");

                html!(self, table {} => {
                    for package in &self.body.packages {
                        html!(self, tr {} => {
                            html!(self, td {class => "package-item"} => {
                                let package_url = self.package_url(package);
                                html!(self, a {class => "name-package", href => package_url} ~
                                        Escape(package.to_string().as_str()));
                            });

                            html!(self, td {class => "package-item-doc"} => {
                                self.doc(::std::iter::empty())?;
                            });
                        });
                    }
                });
            });

            Ok(())
        })
    };
);

impl<'env> IndexProcessor<'env> {}
