//! Backend for Rust

use super::RUST_CONTEXT;
use backend::{CompilerOptions, Environment, ForContext, FromNaming, Naming, PackageUtils,
              SnakeCase};
use backend::errors::*;
use core::{ForEachLoc, RpEnumBody, RpEnumOrdinal, RpField, RpInterfaceBody, RpName, RpServiceBody,
           RpServiceEndpoint, RpTupleBody, RpType, RpTypeBody};
use genco::{Element, IntoTokens, Quoted, Rust, Tokens};
use genco::rust::{imported_alias, imported_alias_ref, imported_ref};
use listeners::Listeners;
use rust_compiler::RustCompiler;
use rust_file_spec::RustFileSpec;
use rust_options::RustOptions;
use std::borrow::Cow;

/// Serializer derives.
pub struct Derives;

impl<'el> IntoTokens<'el, Rust<'el>> for Derives {
    fn into_tokens(self) -> Tokens<'el, Rust<'el>> {
        "#[derive(Serialize, Deserialize, Debug)]".into()
    }
}

/// A serde rename annotation.
pub struct Rename<'el>(&'el str);

impl<'el> IntoTokens<'el, Rust<'el>> for Rename<'el> {
    fn into_tokens(self) -> Tokens<'el, Rust<'el>> {
        toks!["#[serde(rename = ", self.0.quoted(), ")]"]
    }
}

/// Tag attribute.
pub struct Tag<'el>(&'el str);

impl<'el> IntoTokens<'el, Rust<'el>> for Tag<'el> {
    fn into_tokens(self) -> Tokens<'el, Rust<'el>> {
        toks!["#[serde(tag = ", self.0.quoted(), ")]"]
    }
}

const TYPE_SEP: &'static str = "_";
const SCOPE_SEP: &'static str = "::";

pub struct RustBackend {
    pub env: Environment,
    listeners: Box<Listeners>,
    id_converter: Option<Box<Naming>>,
    to_lower_snake: Box<Naming>,
    hash_map: Rust<'static>,
    json_value: Rust<'static>,
    datetime: Option<Tokens<'static, Rust<'static>>>,
}

impl RustBackend {
    pub fn new(
        env: Environment,
        options: RustOptions,
        listeners: Box<Listeners>,
        id_converter: Option<Box<Naming>>,
    ) -> RustBackend {
        RustBackend {
            env: env,
            listeners: listeners,
            id_converter: id_converter,
            to_lower_snake: SnakeCase::new().to_lower_snake(),
            hash_map: imported_ref("std::collections", "HashMap"),
            json_value: imported_alias_ref("serde_json", "Value", "json"),
            datetime: options.datetime.clone(),
        }
    }

    pub fn compiler(&self, options: CompilerOptions) -> Result<RustCompiler> {
        Ok(RustCompiler {
            out_path: options.out_path,
            backend: self,
        })
    }

    pub fn verify(&self) -> Result<()> {
        Ok(())
    }

    fn ident(&self, name: &str) -> String {
        if let Some(ref id_converter) = self.id_converter {
            id_converter.convert(name)
        } else {
            name.to_owned()
        }
    }

    fn convert_type_name(&self, name: &RpName) -> String {
        name.join(TYPE_SEP)
    }

    fn convert_type_id<'el>(&self, name: &'el RpName) -> Result<Element<'el, Rust<'el>>> {
        let registered = self.env.lookup(name)?;

        let local_name = registered.local_name(&name, |p| p.join(TYPE_SEP), |c| c.join(SCOPE_SEP));

        if let Some(ref prefix) = name.prefix {
            let package_name = self.package(&name.package).parts.join("::");
            return Ok(
                imported_alias(
                    Cow::Owned(package_name),
                    Cow::Owned(local_name),
                    Cow::Borrowed(prefix),
                ).into(),
            );
        }

        Ok(local_name.into())
    }

    fn into_type<'el>(&self, field: &'el RpField) -> Result<Tokens<'el, Rust<'el>>> {
        let stmt = self.into_rust_type(&field.ty)?;

        if field.is_optional() {
            return Ok(toks!["Option<", stmt, ">"]);
        }

        Ok(stmt)
    }

    fn enum_value_fn<'el>(
        &self,
        name: String,
        match_body: Tokens<'el, Rust<'el>>,
    ) -> Tokens<'el, Rust<'el>> {
        let mut value_fn = Tokens::new();
        let mut match_decl = Tokens::new();

        match_decl.push("match *self {");
        match_decl.nested(match_body);
        match_decl.push("}");

        value_fn.push("pub fn value(&self) -> &'static str {");
        value_fn.nested(toks!["use self::", name, "::*;"]);
        value_fn.nested(match_decl);
        value_fn.push("}");

        value_fn
    }

    fn datetime<'el>(&self, ty: &RpType) -> Result<Tokens<'el, Rust<'el>>> {
        if let Some(ref datetime) = self.datetime {
            return Ok(datetime.clone().into());
        }

        Err(
            ErrorKind::MissingTypeImpl(ty.clone(), "try: -m chrono").into(),
        )
    }

    pub fn into_rust_type<'el>(&self, ty: &'el RpType) -> Result<Tokens<'el, Rust<'el>>> {
        use self::RpType::*;

        let ty = match *ty {
            String => toks!["String"],
            DateTime => self.datetime(ty)?,
            Bytes => toks!["String"],
            Signed { ref size } => {
                if size.map(|s| s <= 32usize).unwrap_or(true) {
                    toks!["i32"]
                } else {
                    toks!["i64"]
                }
            }
            Unsigned { ref size } => {
                if size.map(|s| s <= 32usize).unwrap_or(true) {
                    toks!["u32"]
                } else {
                    toks!["u64"]
                }
            }
            Float => toks!["f32"],
            Double => toks!["f64"],
            Boolean => toks!["bool"],
            Array { ref inner } => {
                let argument = self.into_rust_type(inner)?;
                toks!["Vec<", argument, ">"]
            }
            Name { ref name } => toks![self.convert_type_id(name)?],
            Map { ref key, ref value } => {
                let key = self.into_rust_type(key)?;
                let value = self.into_rust_type(value)?;
                toks![self.hash_map.clone(), "<", key, ", ", value, ">"]
            }
            Any => toks![self.json_value.clone()],
        };

        Ok(ty)
    }

    // Build the corresponding element out of a field declaration.
    fn field_element<'el>(&self, field: &'el RpField) -> Result<Tokens<'el, Rust<'el>>> {
        let mut elements = Tokens::new();

        let ident = self.ident(field.ident());
        let type_spec = self.into_type(field)?;

        if field.is_optional() {
            elements.push(toks!["#[serde(skip_serializing_if=\"Option::is_none\")]"]);
        }

        if field.name() != ident {
            elements.push(Rename(field.name()));
        }

        elements.push(toks![ident, ": ", type_spec, ","]);

        Ok(elements.into())
    }

    pub fn process_tuple<'el>(
        &self,
        out: &mut RustFileSpec<'el>,
        body: &'el RpTupleBody,
    ) -> Result<()> {
        let mut fields = Tokens::new();

        for field in &body.fields {
            fields.push(self.into_type(field)?);
        }

        let name = self.convert_type_name(&body.name);

        let mut elements = Tokens::new();
        elements.push(Derives);
        elements.push(toks![
            "struct ",
            name,
            "(",
            fields.join(", "),
            ");",
        ]);

        out.0.push(elements);
        Ok(())
    }

    pub fn process_enum<'el>(
        &self,
        out: &mut RustFileSpec<'el>,
        body: &'el RpEnumBody,
    ) -> Result<()> {
        let name = self.convert_type_name(&body.name);

        // variant declarations
        let mut variants = Tokens::new();
        // body of value function
        let mut match_body = Tokens::new();

        body.variants.iter().for_each_loc(|variant| {
            let value = if let RpEnumOrdinal::String(ref s) = variant.ordinal {
                if s != variant.local_name.as_str() {
                    variants.push(Rename(s.as_str()));
                }

                s
            } else {
                variant.local_name.as_str()
            };

            match_body.push(toks![
                variant.local_name.value().as_str(),
                " => ",
                value.quoted(),
                ",",
            ]);

            variants.push(toks![variant.local_name.value().as_str(), ","]);
            Ok(()) as Result<()>
        })?;

        let mut out_enum = Tokens::new();

        out_enum.push(Derives);
        out_enum.push(toks!["pub enum ", name.clone(), " {"]);
        out_enum.nested(variants);
        out_enum.push("}");

        let mut out_impl = Tokens::new();

        out_impl.push(toks!["impl ", name.clone(), " {"]);
        out_impl.nested(self.enum_value_fn(name, match_body));

        // code goes into impl
        for code in body.codes.for_context(RUST_CONTEXT) {
            for line in &code.lines {
                out_impl.nested(line.as_str());
            }
        }

        out_impl.push("}");

        out.0.push(out_enum);
        out.0.push(out_impl);
        Ok(())
    }

    pub fn process_type<'el>(
        &self,
        out: &mut RustFileSpec<'el>,
        body: &'el RpTypeBody,
    ) -> Result<()> {
        let mut fields = Tokens::new();

        for field in &body.fields {
            fields.push(field.as_ref().and_then(|f| self.field_element(f))?);
        }

        let name = self.convert_type_name(&body.name);
        let mut t = Tokens::new();

        t.push(Derives);
        t.push(toks!["pub struct ", name, " {"]);
        t.nested(fields);

        // TODO: clone should not be needed
        for code in body.codes.for_context(RUST_CONTEXT) {
            for line in &code.lines {
                t.nested(toks!(line.as_str()));
            }
        }

        t.push("}");

        out.0.push(t);
        Ok(())
    }

    pub fn process_interface<'el>(
        &self,
        out: &mut RustFileSpec<'el>,
        body: &'el RpInterfaceBody,
    ) -> Result<()> {
        let type_name = body.name.join(TYPE_SEP);
        let mut t = Tokens::new();

        t.push(Derives);
        t.push(Tag("type"));
        t.push(toks!["pub enum ", type_name, " {"]);

        for code in body.codes.for_context(RUST_CONTEXT) {
            for line in &code.lines {
                t.nested(line.as_str());
            }
        }

        let sub_types = body.sub_types.values().map(AsRef::as_ref);

        sub_types.for_each_loc(|s| {
            let mut spec = Tokens::new();

            // TODO: clone should not be needed
            if let Some(ref sub_type_name) = s.names.first() {
                let name = sub_type_name.as_str();

                if name != s.local_name.as_str() {
                    spec.push(Rename(name));
                }
            }

            spec.push(toks![s.local_name.as_str(), " {"]);

            for field in body.fields.iter().chain(s.fields.iter()) {
                spec.nested(self.field_element(field)?);
            }

            spec.push("},");
            t.nested(spec);
            Ok(()) as Result<()>
        })?;

        t.push("}");

        out.0.push(t);
        Ok(())
    }

    /// Generate a base struct for service definitions.
    fn service_struct<'el>(&self, body: &'el RpServiceBody) -> Result<Tokens<'el, Rust<'el>>> {
        let type_name = body.name.join(TYPE_SEP);

        let mut t = Tokens::new();

        t.push(toks!["pub struct ", type_name, " {"]);
        t.push("}");

        Ok(t)
    }

    /// Generate the implementation body for an endpoint
    fn service_endpoint_impl<'el>(
        &self,
        endpoint: &'el RpServiceEndpoint,
    ) -> Result<Tokens<'el, Rust<'el>>> {
        let mut t = Tokens::new();
        t.push("pub fn ");
        Ok(t)
    }

    /// Generate an implementation for service definitions.
    fn service_impl<'el>(&self, body: &'el RpServiceBody) -> Result<Tokens<'el, Rust<'el>>> {
        let type_name = body.name.join(TYPE_SEP);

        let mut b = Tokens::new();

        for endpoint in &body.endpoints {
            b.push(self.service_endpoint_impl(endpoint)?);
        }

        let mut t = Tokens::new();
        t.push(toks!["impl ", type_name, " {"]);
        t.nested(b);
        t.push("}");
        Ok(t)
    }

    pub fn process_service<'el>(
        &self,
        out: &mut RustFileSpec<'el>,
        body: &'el RpServiceBody,
    ) -> Result<()> {
        out.0.push(self.service_struct(body)?);
        out.0.push(self.service_impl(body)?);
        Ok(())
    }
}

impl PackageUtils for RustBackend {}
