use super::errors::*;
use super::scope::Scope;
pub use core::*;
pub use parser::ast::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet, LinkedList, hash_map};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Adds a method for all types that supports conversion into core types.
pub trait IntoModel {
    type Output;

    /// Convert the current type to a model.
    fn into_model(self, scope: &Scope) -> Result<Self::Output>;
}

impl IntoModel for Type {
    type Output = RpType;

    fn into_model(self, scope: &Scope) -> Result<RpType> {
        use self::Type::*;

        let out = match self {
            Double => RpType::Double,
            Float => RpType::Float,
            Signed { size } => RpType::Signed { size: size },
            Unsigned { size } => RpType::Unsigned { size: size },
            Boolean => RpType::Boolean,
            String => RpType::String,
            DateTime => RpType::DateTime,
            Name { name } => RpType::Name { name: name.into_model(scope)? },
            Array { inner } => RpType::Array { inner: inner.into_model(scope)? },
            Map { key, value } => RpType::Map {
                key: key.into_model(scope)?,
                value: value.into_model(scope)?,
            },
            Any => RpType::Any,
            Bytes => RpType::Bytes,
        };

        Ok(out)
    }
}

impl<'input> IntoModel for Decl<'input> {
    type Output = RpDecl;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        use self::Decl::*;

        let s = scope.child(self.name().to_owned());

        let out = match self {
            Type(body) => RpDecl::Type(Rc::new(body.into_model(&s)?)),
            Interface(body) => RpDecl::Interface(Rc::new(body.into_model(&s)?)),
            Enum(body) => RpDecl::Enum(Rc::new(body.into_model(&s)?)),
            Tuple(body) => RpDecl::Tuple(Rc::new(body.into_model(&s)?)),
            Service(body) => RpDecl::Service(Rc::new(body.into_model(&s)?)),
        };

        Ok(out)
    }
}

impl<'input> IntoModel for EnumBody<'input> {
    type Output = RpEnumBody;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        let mut variants: Vec<Rc<Loc<RpEnumVariant>>> = Vec::new();

        let (fields, codes, _options, decls) = members_into_model(scope, self.members)?;

        if fields.len() > 0 {
            return Err("enums can't have fields".into());
        }

        let ty = self.ty.into_model(scope)?;

        let variant_type = if let Some(ty) = ty {
            ty.and_then(|ty| {
                ty.as_enum_type().ok_or_else(
                    || "expected string or absent".into(),
                ) as Result<RpEnumType>
            })?
        } else {
            RpEnumType::Generated
        };

        for variant in self.variants {
            let (variant, pos) = variant.take_pair();

            let variant = (variant, &variant_type).into_model(scope).with_pos(&pos)?;

            if let Some(other) = variants.iter().find(
                |v| *v.local_name == *variant.local_name,
            )
            {
                return Err(
                    ErrorKind::EnumVariantConflict(
                        other.local_name.pos().into(),
                        variant.local_name.pos().into(),
                    ).into(),
                );
            }

            variants.push(Rc::new(Loc::new(variant, pos)));
        }

        Ok(RpEnumBody {
            name: scope.as_name(),
            local_name: self.name.to_string(),
            comment: self.comment.into_iter().map(ToOwned::to_owned).collect(),
            decls: decls,
            variant_type: variant_type,
            variants: variants,
            codes: codes,
        })
    }
}

/// enum value with assigned ordinal
impl<'input, 'a> IntoModel for (EnumVariant<'input>, &'a RpEnumType) {
    type Output = RpEnumVariant;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        let (variant, ty) = self;

        let ordinal = if let Some(argument) = variant.argument.into_model(scope)? {
            if !ty.is_assignable_from(&argument) {
                return Err(
                    format!("unexpected value {}, expected type {}", argument, ty).into(),
                );
            }

            argument.and_then(|value| value.to_ordinal())?
        } else {
            RpEnumOrdinal::Generated
        };

        Ok(RpEnumVariant {
            name: scope.as_name().push(variant.name.to_string()),
            local_name: variant.name.clone().map(str::to_string),
            comment: variant.comment.into_iter().map(ToOwned::to_owned).collect(),
            ordinal: ordinal,
        })
    }
}

impl<'input> IntoModel for Field<'input> {
    type Output = RpField;

    fn into_model(self, scope: &Scope) -> Result<RpField> {
        let name = &self.name;

        let field_as = self.field_as.into_model(scope)?.or_else(|| {
            scope.naming().map(|n| n.convert(name))
        });

        Ok(RpField {
            modifier: self.modifier,
            name: self.name.to_string(),
            comment: self.comment.into_iter().map(ToOwned::to_owned).collect(),
            ty: self.ty.into_model(scope)?,
            field_as: field_as,
        })
    }
}

impl<'input> IntoModel for File<'input> {
    type Output = RpFile;

    fn into_model(self, scope: &Scope) -> Result<RpFile> {
        let options = self.options.into_model(scope)?;

        Ok(RpFile {
            options: options,
            decls: self.decls.into_model(scope)?,
        })
    }
}

impl<'input> IntoModel for InterfaceBody<'input> {
    type Output = RpInterfaceBody;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        use std::collections::btree_map::Entry::*;

        let (fields, codes, _options, decls) = members_into_model(scope, self.members)?;

        let mut sub_types: BTreeMap<String, Rc<Loc<RpSubType>>> = BTreeMap::new();

        for sub_type in self.sub_types {
            let (sub_type, pos) = sub_type.take_pair();
            let sub_type = Rc::new(Loc::new(sub_type.into_model(scope)?, pos));

            // key has to be owned by entry
            let key = sub_type.local_name.clone();

            match sub_types.entry(key) {
                Occupied(entry) => {
                    entry.into_mut().merge(sub_type)?;
                }
                Vacant(entry) => {
                    entry.insert(sub_type);
                }
            }
        }

        Ok(RpInterfaceBody {
            name: scope.as_name(),
            local_name: self.name.to_string(),
            comment: self.comment.into_iter().map(ToOwned::to_owned).collect(),
            decls: decls,
            fields: fields,
            codes: codes,
            sub_types: sub_types,
        })
    }
}

/// Generic implementation for vectors.
impl<T> IntoModel for Loc<T>
where
    T: IntoModel,
{
    type Output = Loc<T::Output>;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        let (value, pos) = self.take_pair();
        Ok(Loc::new(value.into_model(scope)?, pos))
    }
}

/// Generic implementation for vectors.
impl<T> IntoModel for Vec<T>
where
    T: IntoModel,
{
    type Output = Vec<T::Output>;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        let mut out = Vec::new();

        for v in self {
            out.push(v.into_model(scope)?);
        }

        Ok(out)
    }
}

impl<T> IntoModel for Option<T>
where
    T: IntoModel,
{
    type Output = Option<T::Output>;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        if let Some(value) = self {
            return Ok(Some(value.into_model(scope)?));
        }

        Ok(None)
    }
}

impl<T> IntoModel for Box<T>
where
    T: IntoModel,
{
    type Output = Box<T::Output>;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        Ok(Box::new((*self).into_model(scope)?))
    }
}

impl<'a> IntoModel for &'a str {
    type Output = String;

    fn into_model(self, _scope: &Scope) -> Result<Self::Output> {
        Ok(self.to_owned())
    }
}

impl IntoModel for String {
    type Output = String;

    fn into_model(self, _scope: &Scope) -> Result<Self::Output> {
        Ok(self)
    }
}

impl IntoModel for RpPackage {
    type Output = RpPackage;

    fn into_model(self, _scope: &Scope) -> Result<Self::Output> {
        Ok(self)
    }
}

impl IntoModel for Name {
    type Output = RpName;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        use self::Name::*;

        let out = match self {
            Relative { parts } => scope.as_name().extend(parts),
            Absolute { prefix, parts } => {
                let package = if let Some(ref prefix) = prefix {
                    if let Some(package) = scope.lookup_prefix(prefix) {
                        package.clone()
                    } else {
                        return Err(ErrorKind::MissingPrefix(prefix.clone()).into());
                    }
                } else {
                    scope.package()
                };

                RpName {
                    prefix: prefix,
                    package: package,
                    parts: parts,
                }
            }
        };

        Ok(out)
    }
}

impl<'input> IntoModel for (&'input Path, usize, usize) {
    type Output = (PathBuf, usize, usize);

    fn into_model(self, _scope: &Scope) -> Result<Self::Output> {
        Ok((self.0.to_owned(), self.1, self.2))
    }
}

impl<'input> IntoModel for OptionDecl<'input> {
    type Output = RpOptionDecl;

    fn into_model(self, scope: &Scope) -> Result<RpOptionDecl> {
        let decl = RpOptionDecl {
            name: self.name.to_owned(),
            value: self.value.into_model(scope)?,
        };

        Ok(decl)
    }
}

impl<'input> IntoModel for PathSegment<'input> {
    type Output = RpPathSegment;

    fn into_model(self, scope: &Scope) -> Result<RpPathSegment> {
        let out = match self {
            PathSegment::Literal { value } => RpPathSegment::Literal {
                value: value.into_model(scope)?,
            },
            PathSegment::Variable { name, ty } => {
                RpPathSegment::Variable {
                    name: name.into_model(scope)?,
                    ty: ty.into_model(scope)?,
                }
            }
        };

        Ok(out)
    }
}

impl<'input> IntoModel for PathSpec<'input> {
    type Output = RpPathSpec;

    fn into_model(self, scope: &Scope) -> Result<RpPathSpec> {
        Ok(RpPathSpec::new(self.segments.into_model(scope)?))
    }
}

struct ServicePath {
    parent: Option<Rc<RefCell<ServicePath>>>,
    segments: Vec<RpPathSegment>,
}

/// Decode the full path of an endpoint.
fn service_full_path(
    path: Rc<RefCell<ServicePath>>,
    last: Option<Loc<RpPathSpec>>,
) -> Result<Vec<RpPathSegment>> {
    let mut segments = Vec::new();

    let mut current = Some(path);

    while let Some(next) = current {
        let n = next.try_borrow()?;
        segments.extend(n.segments.iter().rev().cloned());
        current = n.parent.clone();
    }

    segments.reverse();

    if let Some(last) = last {
        segments.extend(last.segments.iter().cloned());
    }

    Ok(segments)
}

/// Decode service entries.
fn service_entries<'input>(
    scope: &Scope,
    default_name: Option<Loc<&'input str>>,
    entries: Vec<Loc<ServiceEntry>>,
) -> Result<(Vec<RpServiceAccepts>, Vec<RpServiceReturns>)> {
    use self::ServiceEntry::*;
    use self::hash_map::Entry::*;

    let mut accepts = Vec::new();
    let mut returns = Vec::new();
    let mut seen: HashMap<String, Pos> = HashMap::new();

    for entry in entries {
        let (entry, pos) = entry.take_pair();

        match entry {
            Accepts(acc) => {
                let acc = (default_name.as_ref(), acc).into_model(scope)?;

                // Check for duplicates.
                match seen.entry(acc.name.to_string()) {
                    Occupied(entry) => {
                        return Err(
                            ErrorKind::EndpointConflict(
                                format!("endpoint `{}` already defined", acc.name.as_str()),
                                pos.into(),
                                entry.get().into(),
                            ).into(),
                        );
                    }
                    Vacant(entry) => {
                        entry.insert(pos.clone());
                    }
                }

                accepts.push(acc);
            }
            Returns(ret) => {
                returns.push(ret.into_model(scope)?);
            }
        }
    }

    Ok((accepts, returns))
}

fn parse_mime(mime: Option<Loc<String>>) -> Result<Option<Mime>> {
    if let Some(mime) = mime {
        mime.and_then(|m| {
            m.parse().map_err(|_| "invalid mime type".into()).map(Some)
        })
    } else {
        Ok(None)
    }
}

impl<'input> IntoModel for (Option<&'input Loc<&'input str>>, ServiceAccepts<'input>) {
    type Output = RpServiceAccepts;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        let (default_name, accepts) = self;

        let comment = accepts.comment.into_iter().map(ToOwned::to_owned).collect();

        let name = accepts
            .name
            .or_else(|| default_name.map(Clone::clone))
            .into_model(scope)?;

        let name = name.ok_or_else(|| ErrorKind::MissingName)?;

        Ok(RpServiceAccepts {
            comment: comment,
            ty: accepts.ty.into_model(scope)?,
            mime: parse_mime(accepts.mime)?,
            name: name,
        })
    }
}

impl<'input> IntoModel for ServiceReturns<'input> {
    type Output = RpServiceReturns;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        let comment = self.comment.into_iter().map(ToOwned::to_owned).collect();

        let status = self.status.and_then(|n| {
            n.to_u32().ok_or_else::<Error, _>(
                || "invalid status code".into(),
            )
        })?;

        Ok(RpServiceReturns {
            comment: comment,
            status: status,
            ty: self.ty.into_model(scope)?,
            mime: parse_mime(self.mime)?,
        })
    }
}

impl<'input> IntoModel for ServiceBody<'input> {
    type Output = RpServiceBody;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        let mut endpoints: Vec<RpServiceEndpoint> = Vec::new();

        let root = Rc::new(RefCell::new(ServicePath {
            parent: None,
            segments: vec![],
        }));

        let mut queue = LinkedList::new();

        queue.push_back((root, self.children));

        while let Some((parent, children)) = queue.pop_front() {
            for child in children {
                match child {
                    ServiceNested::Segment { path, children } => {
                        let current = Rc::new(RefCell::new(ServicePath {
                            parent: Some(parent.clone()),
                            segments: path.take().into_model(scope)?.segments,
                        }));

                        queue.push_back((current, children));
                    }
                    ServiceNested::Endpoint {
                        comment,
                        method,
                        path,
                        default_name,
                        options: _options,
                        entries,
                    } => {
                        let path_segments =
                            service_full_path(parent.clone(), path.into_model(scope)?)?;

                        let comment = comment.into_iter().map(ToOwned::to_owned).collect();

                        let (accepts, returns) = service_entries(scope, default_name, entries)?;

                        endpoints.push(RpServiceEndpoint {
                            comment: comment,
                            method: method.into_model(scope)?,
                            path: RpPathSpec::new(path_segments),
                            accepts: accepts,
                            returns: returns,
                        });
                    }
                }
            }
        }

        let comment = self.comment.into_iter().map(ToOwned::to_owned).collect();

        Ok(RpServiceBody {
            name: scope.as_name(),
            local_name: self.name.to_string(),
            comment: comment,
            endpoints: endpoints,
            decls: vec![],
        })
    }
}

impl<'input> IntoModel for SubType<'input> {
    type Output = RpSubType;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        use self::Member::*;

        let mut fields: Vec<Loc<RpField>> = Vec::new();
        let mut codes = Vec::new();
        let mut options = Vec::new();
        let mut decls = Vec::new();

        for member in self.members {
            let (member, pos) = member.take_pair();

            match member {
                Field(field) => {
                    let field = field.into_model(scope)?;

                    if let Some(other) = fields.iter().find(|f| {
                        f.name() == field.name() || f.ident() == field.ident()
                    })
                    {
                        return Err(
                            ErrorKind::FieldConflict(
                                field.ident().to_owned(),
                                pos.into(),
                                other.pos().into(),
                            ).into(),
                        );
                    }

                    fields.push(Loc::new(field, pos));
                }
                Code(context, lines) => {
                    codes.push(code(pos, context.to_owned(), lines));
                }
                Option(option) => {
                    options.push(Loc::new(option.into_model(scope)?, pos));
                }
                InnerDecl(decl) => {
                    decls.push(Rc::new(Loc::new(decl.into_model(scope)?, pos)));
                }
            }
        }

        let names = options.find_all_strings("name")?;

        let comment = self.comment.into_iter().map(ToOwned::to_owned).collect();

        Ok(RpSubType {
            name: scope.as_name().push(self.name.to_string()),
            local_name: self.name.to_string(),
            comment: comment,
            decls: decls,
            fields: fields,
            codes: codes,
            names: names,
        })
    }
}

impl<'input> IntoModel for TupleBody<'input> {
    type Output = RpTupleBody;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        let (fields, codes, _options, decls) = members_into_model(scope, self.members)?;

        Ok(RpTupleBody {
            name: scope.as_name(),
            local_name: self.name.to_string(),
            comment: self.comment.into_iter().map(ToOwned::to_owned).collect(),
            decls: decls,
            fields: fields,
            codes: codes,
        })
    }
}

impl<'input> IntoModel for TypeBody<'input> {
    type Output = RpTypeBody;

    fn into_model(self, scope: &Scope) -> Result<Self::Output> {
        let (fields, codes, options, decls) = members_into_model(scope, self.members)?;

        let reserved: HashSet<Loc<String>> = options
            .find_all_identifiers("reserved")?
            .into_iter()
            .collect();

        Ok(RpTypeBody {
            name: scope.as_name(),
            local_name: self.name.to_string(),
            comment: self.comment.into_iter().map(ToOwned::to_owned).collect(),
            decls: decls,
            fields: fields,
            codes: codes,
            reserved: reserved,
        })
    }
}

type Fields = Vec<Loc<RpField>>;
type Codes = Vec<Loc<RpCode>>;
type OptionVec = Vec<Loc<RpOptionDecl>>;

pub fn code(pos: Pos, context: String, lines: Vec<String>) -> Loc<RpCode> {
    let code = RpCode {
        context: context,
        lines: lines,
    };

    Loc::new(code, pos)
}

pub fn members_into_model(
    scope: &Scope,
    members: Vec<Loc<Member>>,
) -> Result<(Fields, Codes, OptionVec, Vec<Rc<Loc<RpDecl>>>)> {
    use self::Member::*;

    let mut fields: Vec<Loc<RpField>> = Vec::new();
    let mut codes = Vec::new();
    let mut options: Vec<Loc<RpOptionDecl>> = Vec::new();
    let mut decls = Vec::new();

    for member in members {
        let (value, pos) = member.take_pair();

        match value {
            Field(field) => {
                let field = field.into_model(scope)?;

                if let Some(other) = fields.iter().find(|f| {
                    f.name() == field.name() || f.ident() == field.ident()
                })
                {
                    return Err(
                        ErrorKind::FieldConflict(
                            field.ident().to_owned(),
                            pos.into(),
                            other.pos().into(),
                        ).into(),
                    );
                }

                fields.push(Loc::new(field, pos));
            }
            Code(context, lines) => {
                codes.push(code(pos.into(), context.to_owned(), lines));
            }
            Option(option) => {
                options.push(Loc::new(option.into_model(scope)?, pos));
            }
            InnerDecl(decl) => {
                decls.push(Rc::new(Loc::new(decl.into_model(scope)?, pos)));
            }
        }
    }

    Ok((fields, codes, options, decls))
}

impl<'input> IntoModel for Value<'input> {
    type Output = RpValue;

    fn into_model(self, scope: &Scope) -> Result<RpValue> {
        let out = match self {
            Value::String(string) => RpValue::String(string),
            Value::Number(number) => RpValue::Number(number),
            Value::Boolean(boolean) => RpValue::Boolean(boolean),
            Value::Identifier(identifier) => RpValue::Identifier(identifier.to_owned()),
            Value::Array(inner) => RpValue::Array(inner.into_model(scope)?),
        };

        Ok(out)
    }
}
