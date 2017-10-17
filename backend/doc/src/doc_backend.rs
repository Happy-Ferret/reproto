//! Backend module for Documentation.

use super::{DOC_CSS_NAME, EXT, NORMALIZE_CSS_NAME};
use backend::{Environment, PackageUtils};
use backend::errors::*;
use core::{ForEachLoc, Loc, RpEnumBody, RpEnumVariant, RpField, RpInterfaceBody, RpName,
           RpPackage, RpServiceBody, RpServiceEndpoint, RpTupleBody, RpType, RpTypeBody,
           RpVersionedPackage, Version, WithPos};
use doc_builder::{DefaultDocBuilder, DocBuilder};
use doc_collector::{DocCollector, DocDecl};
use doc_listeners::DocListeners;
use doc_options::DocOptions;
use escape::Escape;
use macros::FormatAttribute;
use pulldown_cmark as markdown;
use std::collections::HashMap;
use std::rc::Rc;

pub struct DocBackend {
    pub env: Environment,
    #[allow(dead_code)]
    options: DocOptions,
    listeners: Box<DocListeners>,
    pub theme: String,
    pub themes: HashMap<&'static str, &'static [u8]>,
}

include!(concat!(env!("OUT_DIR"), "/themes.rs"));

fn build_themes() -> HashMap<&'static str, &'static [u8]> {
    let mut m = HashMap::new();

    for (key, value) in build_themes_vec() {
        m.insert(key, value);
    }

    m
}

impl DocBackend {
    pub fn new(
        env: Environment,
        options: DocOptions,
        listeners: Box<DocListeners>,
        theme: String,
    ) -> DocBackend {
        DocBackend {
            env: env,
            options: options,
            listeners: listeners,
            theme: theme,
            themes: build_themes(),
        }
    }

    pub fn verify(&self) -> Result<()> {
        Ok(())
    }

    fn type_url(&self, name: &RpName) -> Result<String> {
        let registered = self.env.lookup(name)?;

        let fragment = registered.local_name(name, |p| p.join("_"), |c| c.join("_"));

        if let Some(_) = name.prefix {
            let package = self.package(&name.package);
            let package = self.package_file(&package);
            return Ok(format!("{}.html#{}", package, fragment));
        }

        return Ok(format!("#{}", fragment));
    }

    fn markdown(input: &str) -> String {
        let p = markdown::Parser::new(input);
        let mut s = String::new();
        markdown::html::push_html(&mut s, p);
        s
    }

    pub fn package_file(&self, package: &RpPackage) -> String {
        package.parts.join("_")
    }

    fn write_markdown<'a, I>(&self, out: &mut DocBuilder, comment: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a String>,
    {
        let mut it = comment.into_iter().peekable();

        if it.peek().is_some() {
            let comment = it.map(ToOwned::to_owned).collect::<Vec<_>>();
            let comment = comment.join("\n");
            html!(out, div { class => "markdown" } ~ Self::markdown(&comment));
        }

        Ok(())
    }

    fn write_docs<'a, I>(&self, out: &mut DocBuilder, comment: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a String>,
    {
        let mut it = comment.into_iter().peekable();

        if it.peek().is_none() {
            return Ok(());
        }

        html!(out, div {class => "docs"} => {
            self.write_markdown(out, it)?;
        });

        Ok(())
    }

    fn write_short_docs<'a, I>(&self, out: &mut DocBuilder, comment: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a String>,
    {
        let mut it = comment.into_iter().peekable();

        if it.peek().is_none() {
            return Ok(());
        }

        html!(out, div {class => "short-docs"} => {
            self.write_markdown(out, it)?;
        });

        Ok(())
    }

    fn write_variants<'b, I>(&self, out: &mut DocBuilder, variants: I) -> Result<()>
    where
        I: IntoIterator<Item = &'b Rc<Loc<RpEnumVariant>>>,
    {
        let mut it = variants.into_iter().peekable();

        if it.peek().is_none() {
            return Ok(());
        }

        html!(out, h2 {} ~ "Variants");

        for variant in it {
            html!(out, div {class => "variant"} => {
                html!(out, div {class => "name"} ~ variant.local_name.as_ref());
                self.write_docs(out, &variant.comment)?;
            });
        }

        Ok(())
    }

    fn write_primitive_type(&self, out: &mut DocBuilder, name: &'static str) -> Result<()> {
        html!(out, span {class => format!("primitive type-{}", name)} => {
            html!(out, code {class => "type-name"} ~ name);
        });

        Ok(())
    }

    fn write_type(&self, out: &mut DocBuilder, ty: &RpType) -> Result<()> {
        use self::RpType::*;

        write!(out, "<span class=\"type\">")?;

        match *ty {
            Double => self.write_primitive_type(out, "double")?,
            Float => self.write_primitive_type(out, "float")?,
            Boolean => self.write_primitive_type(out, "boolean")?,
            String => self.write_primitive_type(out, "string")?,
            DateTime => self.write_primitive_type(out, "datetime")?,
            Bytes => self.write_primitive_type(out, "bytes")?,
            Any => self.write_primitive_type(out, "any")?,
            Signed { ref size } => {
                html!(out, span {class => "type-signed"} => {
                    html!(out, code {class => "type-name"} ~ "signed");

                    if let Some(ref size) = *size {
                        html!(out, span {class => "type-size-sep"} ~ "/");
                        html!(out, span {class => "type-size"} ~ format!("{}", size));
                    }
                });
            }
            Unsigned { ref size } => {
                html!(out, span {class => "type-unsigned"} => {
                    html!(out, code {class => "type-name"} ~ "unsigned");

                    if let Some(ref size) = *size {
                        html!(out, span {class => "type-size-sep"} ~ "/");
                        html!(out, span {class => "type-size"} ~ format!("{}", size));
                    }
                });
            }
            Name { ref name } => {
                let url = self.type_url(name)?;
                let name = name.join("::");

                html!(out, span {class => "type-rp-name"} => {
                    html!(out, a {href => url} ~ name);
                });
            }
            Array { ref inner } => {
                html!(out, span {class => "type-array"} => {
                    html!(out, span {class => "type-array-left"} ~ "[");
                    self.write_type(out, inner)?;
                    html!(out, span {class => "type-array-right"} ~ "]");
                });
            }
            Map { ref key, ref value } => {
                html!(out, span {class => "type-map"} => {
                    html!(out, span {class => "type-map-left"} ~ "{");
                    self.write_type(out, key)?;
                    html!(out, span {class => "type-map-sep"} ~ ":");
                    self.write_type(out, value)?;
                    html!(out, span {class => "type-map-right"} ~ "}");
                });
            }
        }

        write!(out, "</span>")?;
        Ok(())
    }

    fn write_field(&self, out: &mut DocBuilder, field: &RpField) -> Result<()> {
        let mut classes = vec!["field"];

        if field.is_optional() {
            classes.push("optional");
        } else {
            classes.push("required");
        }

        html!(out, div {class => "field"} => {
            let ident = field.ident();
            let name = field.name();

            html!(out, span {class => "field-ident"} ~ ident);

            if field.is_optional() {
                html!(out, span {class => "field-modifier"} ~ "?");
            }

            if name != ident {
                html!(out, span {class => "field-alias"} => {
                    html!(out, span {class => "field-alias-as"} ~ "as");
                    html!(out, code {class => "field-alias-name"} ~ format!("\"{}\"", name));
                });
            }

            self.write_type(out, &field.ty)?;
        });

        self.write_docs(out, &field.comment)?;
        Ok(())
    }

    fn write_fields<'b, I>(&self, out: &mut DocBuilder, fields: I) -> Result<()>
    where
        I: Iterator<Item = &'b Loc<RpField>>,
    {
        html!(out, div {class => "fields"} => {
            html!(out, h2 {} ~ "Fields");

            fields.for_each_loc(|field| {
                self.write_field(out, field)
            })?;
        });

        Ok(())
    }

    fn section_title(&self, out: &mut DocBuilder, ty: &str, name: &str, id: &str) -> Result<()> {
        html!(out, h1 {} => {
            html!(out, span {class => "type"} ~ ty);
            html!(out, a {class => "link", href => format!("#{}", id)} ~ Escape(name));
        });

        Ok(())
    }

    pub fn write_doc<Body>(&self, out: &mut DocBuilder, body: Body) -> Result<()>
    where
        Body: FnOnce(&mut DocBuilder) -> Result<()>,
    {
        html!(out, html {} => {
            html!(out, head {} => {
                html!(@open out, meta {charset => "utf-8"});
                out.new_line()?;

                html!(@open out, meta {
                    name => "viewport",
                    content => "width=device-width, initial-scale=1.0"
                });
                out.new_line()?;

                html!(@open out, link {
                    rel => "stylesheet", type => "text/css", href => NORMALIZE_CSS_NAME
                });
                out.new_line()?;

                html!(@open out, link {
                    rel => "stylesheet", type => "text/css", href => DOC_CSS_NAME
                });
            });

            html!(out, body {} => {
                body(out)?;
            });
        });

        Ok(())
    }

    fn write_endpoint_short(
        &self,
        out: &mut DocBuilder,
        index: usize,
        body: &RpServiceBody,
        endpoint: &RpServiceEndpoint,
    ) -> Result<()> {
        let method = endpoint.method().to_owned();
        let id = format!("{}_{}_{}", body.name, endpoint.id_parts(Self::fragment_filter).join("_"), index);

        html!(out, div {class => format!("endpoint short {}", method.to_lowercase())} => {
            html!(out, a {href => format!("#{}", id)} => {
                html!(out, span {class => "method"} ~ Escape(method.as_ref()));
                html!(out, span {class => "url"} ~ Escape(endpoint.url().as_ref()));
            });

            self.write_short_docs(out, endpoint.comment.iter().take(1))?;
        });

        Ok(())
    }

    fn fragment_filter(url: &str) -> String {
        let mut bytes = [0u8; 4];
        let mut buffer = String::with_capacity(url.len());

        for c in url.chars() {
            let encode = match c {
                'a'...'z' | 'A'...'Z' | '0'...'9' => false,
                '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' => false,
                '-' | '.' | '_' | '~' | ':' | '@' | '/' | '?' => false,
                _ => true,
            };

            if encode {
                let result = c.encode_utf8(&mut bytes);

                for b in result.bytes() {
                    buffer.extend(format!("%{:X}", b).chars());
                }

                continue;
            }

            buffer.push(c);
        }

        buffer
    }

    fn write_endpoint(
        &self,
        out: &mut DocBuilder,
        index: usize,
        body: &RpServiceBody,
        endpoint: &RpServiceEndpoint,
    ) -> Result<()> {
        let method = endpoint.method().to_owned();
        let id = format!("{}_{}_{}", body.name, endpoint.id_parts(Self::fragment_filter).join("_"), index);

        html!(out, div {class => format!("endpoint {}", method.to_lowercase()), id => id} => {
            html!(out, h2 {} => {
                html!(out, span {class => "method"} ~ Escape(method.as_ref()));

                html!(out, a {class => "url", href => format!("#{}", id)}
                    ~ Escape(endpoint.url().as_ref()));
            });

            self.write_docs(out, &endpoint.comment)?;

            if !endpoint.accepts.is_empty() {
                for accept in &endpoint.accepts {
                    html!(out, div {class => "name"} => {
                        let mime = accept.mime
                            .as_ref()
                            .map(|m| format!("{}", m))
                            .unwrap_or("*/*".to_owned());

                        html!(out, span {class => "type"} ~ "Accepts");
                        html!(out, span {class => "name"} ~ accept.name.as_str());
                        html!(out, span {class => "mime"} => {
                            html!(out, code {} ~ Escape(mime.as_ref()));
                        });

                        let (ty, pos) = accept.ty.as_ref_pair();
                        self.write_type(out, ty).with_pos(pos)?;
                    });

                    self.write_docs(out, &accept.comment)?;
                }

            }

            if !endpoint.returns.is_empty() {
                for returns in &endpoint.returns {
                    html!(out, div {class => "name"} => {
                        let status = returns.status.to_string();

                        let mime = returns.mime
                            .as_ref()
                            .map(|m| format!("{}", m))
                            .unwrap_or("*/*".to_owned());

                        html!(out, span {class => "type"} ~ "Returns");
                        html!(out, span {class => "status"} ~ status);
                        html!(out, span {class => "mime"} => {
                            html!(out, code {} ~ Escape(mime.as_ref()));
                        });

                        let (ty, pos) = returns.ty.as_ref_pair();
                        self.write_type(out, ty).with_pos(pos)?;
                    });

                    self.write_docs(out, &returns.comment)?;
                }
            }
        });

        Ok(())
    }

    /// Write a packages index.
    ///
    /// * `current` if some value indicates which the current package is.
    pub fn write_packages(
        &self,
        out: &mut DocBuilder,
        packages: &[RpVersionedPackage],
        current: Option<&RpVersionedPackage>,
    ) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        html!(out, section {class => "packages"} => {
            html!(out, h2 {} ~ "Packages");

            html!(out, ul {class => "packages-list"} => {
                for package in packages {
                    let name = format!("{}", package);

                    if let Some(current) = current {
                        if package == current {
                            html!(out, li {} ~ format!("<b>{}</b>", Escape(name.as_ref())));
                            continue;
                        }
                    }

                    let package = self.package(package);
                    let url = format!("{}.{}", self.package_file(&package), EXT);

                    html!(out, li {} => {
                        html!(out, a {href => url} ~ Escape(name.as_ref()));
                    });
                }
            });
        });

        Ok(())
    }

    pub fn write_service_overview(
        &self,
        out: &mut DocBuilder,
        service_bodies: Vec<&RpServiceBody>,
    ) -> Result<()> {
        if service_bodies.is_empty() {
            return Ok(());
        }

        html!(out, section {class => "service-overview"} => {
            for body in service_bodies {
                html!(out, h4 {} ~ &body.name);

                self.write_short_docs(out, body.comment.iter().take(1))?;

                for (index, endpoint) in body.endpoints.iter().enumerate() {
                    self.write_endpoint_short(out, index, &body, endpoint)?;
                }
            }
        });

        Ok(())
    }

    pub fn write_types_overview(&self, out: &mut DocBuilder, decls: Vec<DocDecl>) -> Result<()> {
        if decls.is_empty() {
            return Ok(());
        }

        html!(out, section {class => "types-overview"} => {
            html!(out, h2 {} ~ "Types");

            for decl in decls {
                let href = format!("#{}", decl.local_name());

                html!(out, h4 {} => {
                    html!(out, a {href => href} ~ decl.local_name());
                });

                self.write_short_docs(out, decl.comment().iter().take(1))?;
            }
        });

        Ok(())
    }

    pub fn process_service<'p>(
        &self,
        out: &mut DocCollector<'p>,
        body: &'p RpServiceBody,
    ) -> Result<()> {
        let mut new_service = out.new_service(body);
        let mut out = DefaultDocBuilder::new(&mut new_service);

        let title_text = body.name.join("::");
        let id = body.name.join("_");

        html!(out, section {id => &id, class => "service"} => {
            self.section_title(&mut out, "Service", &title_text, &id)?;

            html!(out, div {class => "body"} => {
                self.write_docs(&mut out, &body.comment)?;
            });

            for (index, endpoint) in body.endpoints.iter().enumerate() {
                self.write_endpoint(&mut out, index, body, endpoint)?;
            }
        });

        Ok(())
    }

    pub fn process_enum<'p>(&self, out: &mut DocCollector<'p>, body: &'p RpEnumBody) -> Result<()> {
        let mut new_enum = out.new_type(DocDecl::Enum(body));
        let mut out = DefaultDocBuilder::new(&mut new_enum);

        let title_text = body.name.join("::");
        let id = body.name.join("_");

        html!(out, section {id => &id, class => "enum"} => {
            self.section_title(&mut out, "Enum", &title_text, &id)?;

            html!(out, div {class => "body"} => {
                self.write_docs(&mut out, &body.comment)?;
                self.write_variants(&mut out, body.variants.iter())?;
            });
        });

        Ok(())
    }

    pub fn process_interface<'p>(
        &self,
        out: &mut DocCollector<'p>,
        body: &'p RpInterfaceBody,
    ) -> Result<()> {
        let mut new_interface = out.new_type(DocDecl::Interface(body));
        let mut out = DefaultDocBuilder::new(&mut new_interface);

        let title_text = body.name.join("::");
        let id = body.name.join("_");

        html!(out, section {id => &id, class => "interface"} => {
            self.section_title(&mut out, "Interface", &title_text, &id)?;

            self.write_markdown(&mut out, &body.comment)?;

            if !body.sub_types.is_empty() {
                html!(out, div {class => "sub-types"} => {
                    for sub_type in body.sub_types.values() {
                        let id = format!("{}_{}", body.name, sub_type.name);

                        html!(out, h2 {id => id, class => "sub-type-title"} => {
                            html!(out, a {class => "link", href => format!("#{}", id)} ~
                                    sub_type.local_name);
                        });

                        self.write_markdown(&mut out, &body.comment)?;

                        let fields = body.fields.iter().chain(sub_type.fields.iter());
                        self.write_fields(&mut out, fields)?;
                    }
                });
            }
        });

        Ok(())
    }

    pub fn process_type<'p>(&self, out: &mut DocCollector<'p>, body: &'p RpTypeBody) -> Result<()> {
        let mut new_type = out.new_type(DocDecl::Type(body));
        let mut out = DefaultDocBuilder::new(&mut new_type);

        let title_text = body.name.join("::");
        let id = body.name.join("_");

        html!(out, section {id => &id, class => "type"} => {
            self.section_title(&mut out, "Type", &title_text, &id)?;
            self.write_markdown(&mut out, &body.comment)?;
            self.write_fields(&mut out, body.fields.iter())?;
        });

        Ok(())
    }

    pub fn process_tuple<'p>(
        &self,
        out: &mut DocCollector<'p>,
        body: &'p RpTupleBody,
    ) -> Result<()> {
        let mut new_tuple = out.new_type(DocDecl::Tuple(body));
        let mut out = DefaultDocBuilder::new(&mut new_tuple);

        let id = body.name.join("_");
        let title_text = body.name.join("::");

        html!(out, section {id => &id, class => "tuple"} => {
            self.section_title(&mut out, "Tuple", &title_text, &id)?;
            self.write_markdown(&mut out, &body.comment)?;
            self.write_fields(&mut out, body.fields.iter())?;
        });

        Ok(())
    }
}

impl PackageUtils for DocBackend {
    fn version_package(input: &Version) -> String {
        format!("{}", input).replace(Self::package_version_unsafe, "_")
    }
}
