//! Processor for service declarations.

use backend::Environment;
use backend::errors::*;
use core::RpTypeBody;
use doc_builder::DocBuilder;
use macros::FormatAttribute;
use processor::Processor;

define_processor!(TypeProcessor, RpTypeBody, self {
    self.write_doc(|| {
        let id = self.body.name.join("_");

        html!(self, section {id => &id, class => "section-content section-type"} => {
            self.section_title("type", &self.body.name)?;

            html!(self, div {class => "section-body"} => {
                self.description(&self.body.comment)?;
                self.fields(self.body.fields.iter())?;
            });
        });

        Ok(())
    })
});

impl<'p> TypeProcessor<'p> {}
