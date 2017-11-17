//! Processor for service declarations.

use backend::Environment;
use backend::errors::*;
use core::RpTupleBody;
use doc_builder::DocBuilder;
use macros::FormatAttribute;
use processor::Processor;

define_processor!(TupleProcessor, RpTupleBody, self {
    self.write_doc(|| {
        let id = self.body.name.join("_");

        html!(self, section {id => &id, class => "section-content section-tuple"} => {
            self.section_title("tuple", &self.body.name)?;

            html!(self, div {class => "section-body"} => {
                self.description(&self.body.comment)?;
                self.fields(self.body.fields.iter())?;
            });
        });

        Ok(())
    })
});

impl<'p> TupleProcessor<'p> {}
