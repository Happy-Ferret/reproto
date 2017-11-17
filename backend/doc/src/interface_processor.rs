//! Processor for service declarations.

use backend::Environment;
use backend::errors::*;
use core::RpInterfaceBody;
use doc_builder::DocBuilder;
use macros::FormatAttribute;
use processor::Processor;

define_processor!(InterfaceProcessor, RpInterfaceBody, self {
    self.write_doc(|| {
        let id = self.body.name.join("_");

        html!(self, section {id => &id, class => "section-content section-interface"} => {
            self.section_title("interface", &self.body.name)?;

            self.description(&self.body.comment)?;

            for sub_type in self.body.sub_types.values() {
                let id = sub_type.name.join("_");

                html!(self, h2 {id => id, class => "sub-type-title"} => {
                    html!(self, span {class => "kind"} ~ "subtype");
                    self.full_name(&sub_type.name, Some(&self.body.name))?;
                });

                self.description(&self.body.comment)?;

                let fields = self.body.fields.iter().chain(sub_type.fields.iter());
                self.fields(fields)?;
            }
        });

        Ok(())
    })
});

impl<'p> InterfaceProcessor<'p> {}
