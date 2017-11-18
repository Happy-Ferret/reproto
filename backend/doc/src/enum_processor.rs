//! Processor for service declarations.

use backend::Environment;
use backend::errors::*;
use core::{Loc, RpEnumBody, RpVariant};
use doc_builder::DocBuilder;
use escape::Escape;
use macros::FormatAttribute;
use processor::Processor;
use std::rc::Rc;

define_processor!(EnumProcessor, RpEnumBody, self,
    process => {
        self.write_doc(|| {
            let id = self.body.name.join("_");

            html!(self, section {id => &id, class => "section-content section-enum"} => {
                self.section_title("enum", &self.body.name)?;
                self.doc(&self.body.comment)?;
                self.variants(self.body.variants.iter())?;
            });

            Ok(())
        })
    };
);

impl<'p> EnumProcessor<'p> {
    fn variants<'b, I>(&self, variants: I) -> Result<()>
    where
        I: IntoIterator<Item = &'b Rc<Loc<RpVariant>>>,
    {
        let mut it = variants.into_iter().peekable();

        if it.peek().is_none() {
            return Ok(());
        }

        for variant in it {
            html!(self, h3 {} => {
                html!(self, span {class => "kind"} ~ "variant");
                self.full_name_without_package(&variant.name)?;
                html!(self, span {class => "keyword"} ~ "as");
                html!(self, span {class => "variant-ordinal"} ~
                      Escape(format!("\"{}\"", variant.ordinal()).as_str()));
            });

            self.doc(&variant.comment)?;
        }

        Ok(())
    }
}
