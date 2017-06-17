/// Module that adds fasterxml annotations to generated classes.
use codeviz::java::*;
use super::converter::Converter;
use super::decode::Decode;
use super::match_decode::MatchDecode;
use super::models::*;
use super::processor::*;
use super::super::environment::Environment;
use super::value_builder::ValueBuilder;

pub struct Module {
    override_: ClassType,
    creator: ClassType,
    value: ClassType,
    property: ClassType,
    sub_types: ClassType,
    type_info: ClassType,
    serialize: ClassType,
    deserialize: ClassType,
    deserializer: ClassType,
    serializer: ClassType,
    generator: ClassType,
    serializer_provider: ClassType,
    parser: ClassType,
    deserialization_context: ClassType,
    token: ClassType,
    string: ClassType,
    io_exception: ClassType,
}

impl Module {
    pub fn new() -> Module {
        Module {
            override_: Type::class("java.lang", "Override"),
            creator: Type::class("com.fasterxml.jackson.annotation", "JsonCreator"),
            value: Type::class("com.fasterxml.jackson.annotation", "JsonValue"),
            property: Type::class("com.fasterxml.jackson.annotation", "JsonProperty"),
            sub_types: Type::class("com.fasterxml.jackson.annotation", "JsonSubTypes"),
            type_info: Type::class("com.fasterxml.jackson.annotation", "JsonTypeInfo"),
            serialize: Type::class("com.fasterxml.jackson.databind.annotation", "JsonSerialize"),
            deserialize: Type::class("com.fasterxml.jackson.databind.annotation",
                                     "JsonDeserialize"),
            serializer: Type::class("com.fasterxml.jackson.databind", "JsonSerializer"),
            deserializer: Type::class("com.fasterxml.jackson.databind", "JsonDeserializer"),
            generator: Type::class("com.fasterxml.jackson.core", "JsonGenerator"),
            serializer_provider: Type::class("com.fasterxml.jackson.databind",
                                             "SerializerProvider"),
            parser: Type::class("com.fasterxml.jackson.core", "JsonParser"),
            deserialization_context: Type::class("com.fasterxml.jackson.databind",
                                                 "DeserializationContext"),
            token: Type::class("com.fasterxml.jackson.core", "JsonToken"),
            string: Type::class("java.lang", "String"),
            io_exception: Type::class("java.io", "IOException"),
        }
    }

    /// RpName serialize implementation for tuples.
    fn tuple_serializer(&self,
                        fields: &Vec<JavaField>,
                        class_type: &ClassType)
                        -> Result<ClassSpec> {
        let mut serializer = ClassSpec::new(mods![Modifier::Public, Modifier::Static],
                                            "Serializer");

        serializer.extends(self.serializer.with_arguments(vec![&class_type]));

        let value = ArgumentSpec::new(mods![Modifier::Final], &class_type, "value");
        let jgen = ArgumentSpec::new(mods![Modifier::Final], &self.generator, "jgen");
        let provider = ArgumentSpec::new(mods![Modifier::Final],
                                         &self.serializer_provider,
                                         "provider");

        let mut serialize = MethodSpec::new(mods![Modifier::Public], "serialize");
        serialize.throws(&self.io_exception);
        serialize.push_argument(&value);
        serialize.push_argument(&jgen);
        serialize.push_argument(&provider);
        serialize.push_annotation(&self.override_);

        let mut body = Elements::new();
        body.push(stmt![&jgen, ".writeStartArray();"]);

        for field in fields {
            let field_stmt = stmt![&value, ".", &field.java_spec];

            let write = match field.java_type {
                Type::Primitive(ref primitive) => {
                    match *primitive {
                        SHORT | LONG | INTEGER | FLOAT | DOUBLE => {
                            stmt!["writeNumber(", field_stmt, ")"]
                        }
                        _ => return Err("cannot serialize type".into()),
                    }
                }
                Type::Class(ref class) => {
                    if *class == self.string {
                        stmt!["writeString(", field_stmt, ")"]
                    } else {
                        stmt!["writeObject(", field_stmt, ")"]
                    }
                }
                _ => stmt!["writeObject(", field_stmt, ")"],
            };

            body.push(stmt![&jgen, ".", write, ";"]);
        }

        body.push(stmt![&jgen, ".writeEndArray();"]);

        serialize.push(body);

        serializer.push(serialize);

        Ok(serializer)
    }

    fn deserialize_method_for_type<A>(&self,
                                      ty: &Type,
                                      parser: &A)
                                      -> Result<(Option<(Statement, &str)>, Statement)>
        where A: Into<Variable> + Clone
    {
        match *ty {
            Type::Primitive(ref primitive) => {
                let test = stmt!["!", parser, ".nextToken().isNumeric()"];

                match *primitive {
                    SHORT => {
                        Ok((Some((test, "VALUE_NUMBER_INT")), stmt![parser, ".getShortValue()"]))
                    }
                    LONG => {
                        Ok((Some((test, "VALUE_NUMBER_INT")), stmt![parser, ".getLongValue()"]))
                    }
                    INTEGER => {
                        Ok((Some((test, "VALUE_NUMBER_INT")), stmt![parser, ".getIntegerValue()"]))
                    }
                    FLOAT => {
                        Ok((Some((test, "VALUE_NUMBER_FLOAT")), stmt![parser, ".getFloatValue()"]))
                    }
                    DOUBLE => {
                        Ok((Some((test, "VALUE_NUMBER_FLOAT")), stmt![parser, ".getDoubleValue()"]))
                    }
                    _ => return Err("cannot deserialize type".into()),
                }
            }
            Type::Class(ref class) => {
                if *class == self.string {
                    let test = stmt![parser, ".nextToken() != ", &self.token, ".VALUE_STRING"];
                    let token = Some((test, "VALUE_STRING"));
                    return Ok((token, stmt![parser, ".getText()"]));
                }

                if class.arguments.is_empty() {
                    return Ok((None, stmt![parser, ".readValueAs(", class, ".class)"]));
                }

                // TODO: support generics
                return Err("cannot deserialize type".into());
            }
            Type::Local(ref local) => {
                return Ok((None, stmt![parser, ".readValueAs(", &local.name, ")"]));
            }
        }
    }

    fn wrong_token_exception(&self,
                             ctxt: &ArgumentSpec,
                             parser: &ArgumentSpec,
                             token: &str)
                             -> Statement {
        let mut arguments = Statement::new();
        arguments.push(parser);
        arguments.push(stmt![&self.token, ".", token]);
        arguments.push("null");

        stmt!["throw ", ctxt, ".wrongTokenException(", arguments.join(", "), ");"]
    }

    /// RpName deserialize implementation for tuples.
    fn tuple_deserializer(&self,
                          fields: &Vec<JavaField>,
                          class_type: &ClassType)
                          -> Result<ClassSpec> {
        let mut deserializer = ClassSpec::new(mods![Modifier::Public, Modifier::Static],
                                              "Deserializer");

        deserializer.extends(self.deserializer.with_arguments(vec![&class_type]));

        let parser = ArgumentSpec::new(mods![Modifier::Final], &self.parser, "parser");
        let ctxt = ArgumentSpec::new(mods![Modifier::Final],
                                     &self.deserialization_context,
                                     "ctxt");

        let mut deserialize = MethodSpec::new(mods![Modifier::Public], "deserialize");
        deserialize.throws(&self.io_exception);
        deserialize.push_argument(&parser);
        deserialize.push_argument(&ctxt);
        deserialize.push_annotation(&self.override_);
        deserialize.returns(&class_type);

        let current_token = stmt![&parser, ".getCurrentToken()"];

        let mut start_array = Elements::new();
        start_array.push(stmt!["if (", &current_token, " != ", &self.token, ".START_ARRAY) {"]);
        start_array.push_nested(self.wrong_token_exception(&ctxt, &parser, "START_ARRAY"));
        start_array.push("}");
        deserialize.push(start_array);

        let mut arguments = Statement::new();

        for field in fields {
            let (token, reader) = self.deserialize_method_for_type(&field.java_type, &parser)?;

            if let Some((test, expected)) = token {
                let mut field_check = Elements::new();
                field_check.push(stmt!["if (", &test, ") {"]);
                field_check.push_nested(self.wrong_token_exception(&ctxt, &parser, expected));
                field_check.push("}");
                deserialize.push(field_check);
            }

            let variable = stmt!["v_", &field.java_spec.name];
            let assign = stmt!["final ", &field.java_spec.ty, " ", &variable, " = ", reader, ";"];
            deserialize.push(assign);
            arguments.push(variable);
        }

        let next_token = stmt![&parser, ".nextToken()"];

        let mut end_array = Elements::new();
        end_array.push(stmt!["if (", &next_token, " != ", &self.token, ".END_ARRAY) {"]);
        end_array.push_nested(self.wrong_token_exception(&ctxt, &parser, "END_ARRAY"));
        end_array.push("}");
        deserialize.push(end_array);

        deserialize.push(stmt!["return new ", &class_type, "(", arguments.join(", "), ");"]);

        deserializer.push(deserialize);
        Ok(deserializer)
    }

    fn add_class_annotations(&self, spec: &mut ClassSpec, fields: &Vec<JavaField>) -> Result<()> {
        if spec.constructors.len() != 1 {
            return Err("Expected exactly one constructor".into());
        }

        let constructor = &mut spec.constructors[0];

        if constructor.arguments.len() != fields.len() {
            return Err(format!("the number of constructor arguments ({}) did not match \
                                the number of fields ({})",
                               constructor.arguments.len(),
                               fields.len())
                .into());
        }

        let creator_annotation = AnnotationSpec::new(&self.creator);
        constructor.push_annotation(&creator_annotation);

        let zipped = constructor.arguments.iter_mut().zip(fields.iter());

        for (argument, field) in zipped {
            let mut property = AnnotationSpec::new(&self.property);
            property.push_argument(stmt![Variable::String(field.name.to_owned())]);
            argument.push_annotation(&property);
        }

        Ok(())
    }

    fn build_model(&self, fields: &Vec<JavaField>) -> ClassSpec {
        let mut model = ClassSpec::new(mods![Modifier::Public, Modifier::Static], "Model");

        for field in fields {
            model.push_field(&field.java_spec);
        }

        let mut constructor = ConstructorSpec::new(mods![Modifier::Public]);

        for field in fields {
            let argument =
                ArgumentSpec::new(mods![Modifier::Final], &field.java_spec.ty, &field.ident);
            constructor.push_argument(&argument);
            constructor.push(stmt!["this.", &field.ident, " = ", argument, ";"]);
        }

        model.push_constructor(constructor);

        model
    }

    /// Build the fallback method used when matching to deserialize using a companion model.
    fn deserialize_using_model(&self,
                               model: &ClassSpec,
                               class_type: &ClassType,
                               parser: &ArgumentSpec)
                               -> Result<Elements> {
        let mut elements = Elements::new();

        // fallback to deserializing model and assigning fields from it.
        let model_name = stmt!["m"];

        let var_decl = stmt!["final ", &model.name, " ", &model_name];
        elements.push(stmt![var_decl, " = ", parser, ".readValueAs(", &model.name, ".class);"]);

        let mut arguments = Statement::new();

        for field in &model.fields {
            arguments.push(stmt![&model_name, ".", &field.name]);
        }

        elements.push(stmt!["return new ", class_type, "(", arguments.join(", "), ");"]);

        Ok(elements)
    }

    /// Build a match deserializer.
    ///
    /// * `match_decl` - The declaration to to use when deserializing.
    /// * `model` - The model to return when falling back to default deserialization.
    /// * `class_type` - The class to deserialize to.
    fn match_deserializer(&self,
                          type_id: &RpTypeId,
                          match_decl: &RpMatchDecl,
                          model: &ClassSpec,
                          class_type: &ClassType,
                          processor: &Processor)
                          -> Result<ClassSpec> {
        // TODO: mostly common code for setting up a deserializer with tuple_deserializer.
        let mut deserializer = ClassSpec::new(mods![Modifier::Public, Modifier::Static],
                                              "Deserializer");

        deserializer.extends(self.deserializer.with_arguments(vec![&class_type]));

        let parser = ArgumentSpec::new(mods![Modifier::Final], &self.parser, "parser");
        let ctxt = ArgumentSpec::new(mods![Modifier::Final],
                                     &self.deserialization_context,
                                     "ctxt");

        let mut deserialize = MethodSpec::new(mods![Modifier::Public], "deserialize");
        deserialize.throws(&self.io_exception);
        deserialize.push_argument(&parser);
        deserialize.push_argument(&ctxt);
        deserialize.push_annotation(&self.override_);
        deserialize.returns(&class_type);

        let match_decode = FasterXmlMatchDecode {
            processor: processor,
            module: self,
        };

        let p = stmt![&parser];

        if let Some(by_value) = match_decode.decode_by_value(type_id, match_decl, &p)? {
            deserialize.push(by_value.join(Spacing));
        }

        if let Some(by_type) = match_decode.decode_by_type(type_id, match_decl, &p)? {
            deserialize.push(by_type.join(Spacing));
        }

        deserialize.push(self.deserialize_using_model(model, class_type, &parser)?);

        deserializer.push(deserialize);
        Ok(deserializer)
    }

    fn handle_match_decl(&self, event: &mut ClassAdded) -> Result<()> {
        // Add inner model used specifically for regular serialization.
        let mut model = self.build_model(event.fields);
        self.add_class_annotations(&mut model, &event.fields)?;

        let deserializer = self.match_deserializer(&event.type_id,
                                &event.match_decl,
                                &model,
                                &event.class_type,
                                &event.processor)?;

        let deserializer_type =
            Type::class(&event.class_type.package,
                        &format!("{}.{}", &event.class_type.name, &deserializer.name));

        let mut deserialize_annotation: AnnotationSpec = self.deserialize.clone().into();
        deserialize_annotation.push_argument(stmt!["using = ", deserializer_type, ".class"]);

        event.spec.push_annotation(deserialize_annotation);
        event.spec.push(model);
        event.spec.push(deserializer);

        Ok(())
    }
}

impl Listeners for Module {
    fn class_added(&self, event: &mut ClassAdded) -> Result<()> {
        if !event.match_decl.is_empty() {
            self.handle_match_decl(event)?;
            return Ok(());
        }

        self.add_class_annotations(&mut event.spec, &event.fields)?;
        Ok(())
    }

    fn tuple_added(&self, event: &mut TupleAdded) -> Result<()> {
        let serializer = self.tuple_serializer(&event.fields, &event.class_type)?;

        let serializer_type =
            Type::class(&event.class_type.package,
                        &format!("{}.{}", event.class_type.name, serializer.name));

        let mut serialize_annotation: AnnotationSpec = self.serialize.clone().into();
        serialize_annotation.push_argument(stmt!["using = ", serializer_type, ".class"]);

        event.spec.push_annotation(serialize_annotation);
        event.spec.push(serializer);

        let deserializer = self.tuple_deserializer(&event.fields, &event.class_type)?;

        let deserializer_type =
            Type::class(&event.class_type.package,
                        &format!("{}.{}", &event.class_type.name, deserializer.name));

        let mut deserialize_annotation: AnnotationSpec = self.deserialize.clone().into();
        deserialize_annotation.push_argument(stmt!["using = ", deserializer_type, ".class"]);

        event.spec.push_annotation(deserialize_annotation);
        event.spec.push(deserializer);

        Ok(())
    }

    fn enum_added(&self, event: &mut EnumAdded) -> Result<()> {
        if let Some(ref mut from_value) = *event.from_value {
            from_value.push_annotation(&self.creator);
        }

        if let Some(ref mut to_value) = *event.to_value {
            to_value.push_annotation(&self.value);
        }

        Ok(())
    }

    fn interface_added(&self, event: &mut InterfaceAdded) -> Result<()> {
        {
            let mut arguments = Statement::new();

            arguments.push(stmt!["use=", &self.type_info, ".Id.NAME"]);
            arguments.push(stmt!["include=", &self.type_info, ".As.PROPERTY"]);
            arguments.push(stmt!["property=", Variable::String("type".to_owned())]);

            let mut type_info = AnnotationSpec::new(&self.type_info);
            type_info.push_argument(arguments.join(", "));

            event.spec.push_annotation(&type_info);
        }

        {
            let mut arguments = Statement::new();

            for (key, sub_type) in &event.interface.sub_types {
                for name in &sub_type.names {
                    let name: String = name.as_ref().to_owned();

                    let mut type_args = Statement::new();

                    type_args.push(stmt!["name=", Variable::String(name)]);
                    type_args.push(stmt!["value=", &event.spec.name, ".", key, ".class"]);

                    let a = stmt!["@", &self.sub_types, ".Type(", type_args.join(", "), ")"];

                    arguments.push(a);
                }
            }

            let mut sub_types = AnnotationSpec::new(&self.sub_types);
            sub_types.push_argument(stmt!["{", arguments.join(", "), "}"]);

            event.spec.push_annotation(&sub_types);
        }

        Ok(())
    }
}

struct FasterXmlMatchDecode<'a> {
    processor: &'a Processor,
    module: &'a Module,
}

impl<'a> FasterXmlMatchDecode<'a> {
    fn type_check(&self, data: &Statement, kind: &RpMatchKind) -> Statement {
        match *kind {
            RpMatchKind::Any => stmt!["true"],
            RpMatchKind::Object => {
                stmt![stmt![data, ".getCurrentToken() == ", &self.module.token, ".START_OBJECT"]]
            }
            RpMatchKind::Array => {
                stmt![data, ".getCurrentToken() == ", &self.module.token, ".START_ARRAY"]
            }
            RpMatchKind::String => {
                stmt![data, ".getCurrentToken() == ", &self.module.token, ".VALUE_STRING"]
            }
            RpMatchKind::Boolean => {
                stmt![data, ".getCurrentToken() == ", &self.module.token, ".VALUE_BOOLEAN"]
            }
            RpMatchKind::Number => stmt![data, ".getCurrentToken().isNumeric()"],
        }
    }

    fn value_check(&self,
                   data: &Statement,
                   kind: &RpMatchKind,
                   other: &Statement)
                   -> Result<Statement> {
        match *kind {
            RpMatchKind::String => return Ok(stmt![data, ".getText() == ", &other]),
            RpMatchKind::Boolean => return Ok(stmt![data, ".getBooleanValue() == ", &other]),
            RpMatchKind::Number => return Ok(stmt![data, ".getLongValue() == ", &other]),
            _ => {}
        }

        Err("not supported".into())
    }
}

impl<'a> Decode for FasterXmlMatchDecode<'a> {
    fn decode(&self,
              type_id: &RpTypeId,
              pos: &RpPos,
              ty: &RpType,
              input: &Self::Stmt)
              -> Result<Self::Stmt> {
        let ty = self.processor.into_java_type(pos, &type_id.package, ty)?;
        let (_, reader) = self.module.deserialize_method_for_type(&ty, input)?;
        Ok(reader)
    }
}

impl<'a> MatchDecode for FasterXmlMatchDecode<'a> {
    fn match_value(&self,
                   data: &Statement,
                   value: &RpValue,
                   value_stmt: Statement,
                   _result: &RpValue,
                   result_stmt: Statement)
                   -> Result<Elements> {
        let mut value_body = Elements::new();
        let kind = value.as_match_kind();
        let check = self.type_check(data, &kind);
        let compare_value = self.value_check(data, &kind, &value_stmt)?;

        value_body.push(stmt!["if (", check, " && ", compare_value, ") {"]);
        value_body.push_nested(stmt!["return ", result_stmt, ";"]);
        value_body.push("}");

        Ok(value_body)
    }

    fn match_type(&self,
                  type_id: &RpTypeId,
                  data: &Statement,
                  kind: &RpMatchKind,
                  variable: &str,
                  decode: Statement,
                  result: Statement,
                  value: &RpByTypeMatch)
                  -> Result<Elements> {
        let variable_ty = self.processor
            .into_java_type(value.variable.pos(), &type_id.package, &value.variable.ty)?;

        let mut value_body = Elements::new();
        let check = self.type_check(data, kind);

        value_body.push(stmt!["if (", check, ") {"]);
        value_body.push_nested(stmt!["final ", &variable_ty, " ", &variable, " = ", decode, ";"]);
        value_body.push_nested(stmt!["return ", &result, ";"]);
        value_body.push("}");

        Ok(value_body)
    }
}

impl<'a> Converter for FasterXmlMatchDecode<'a> {
    type Type = Type;
    type Stmt = Statement;
    type Elements = Elements;
    type Variable = Variable;

    fn new_var(&self, name: &str) -> Self::Stmt {
        stmt![name]
    }

    fn convert_type(&self, pos: &RpPos, type_id: &RpTypeId) -> Result<Type> {
        self.processor.convert_type(pos, type_id)
    }
}

impl<'a> ValueBuilder for FasterXmlMatchDecode<'a> {
    fn env(&self) -> &Environment {
        self.processor.env()
    }

    fn identifier(&self, identifier: &str) -> Result<Self::Stmt> {
        self.processor.identifier(identifier)
    }

    fn optional_empty(&self) -> Result<Self::Stmt> {
        self.processor.optional_empty()
    }

    fn constant(&self, ty: Self::Type) -> Result<Self::Stmt> {
        self.processor.constant(ty)
    }

    fn instance(&self, ty: Self::Type, arguments: Vec<Self::Stmt>) -> Result<Self::Stmt> {
        self.processor.instance(ty, arguments)
    }

    fn number(&self, number: &RpNumber) -> Result<Self::Stmt> {
        self.processor.number(number)
    }

    fn signed(&self, number: &RpNumber, size: &Option<usize>) -> Result<Self::Stmt> {
        self.processor.signed(number, size)
    }

    fn unsigned(&self, number: &RpNumber, size: &Option<usize>) -> Result<Self::Stmt> {
        self.processor.unsigned(number, size)
    }

    fn float(&self, number: &RpNumber) -> Result<Self::Stmt> {
        self.processor.float(number)
    }

    fn double(&self, number: &RpNumber) -> Result<Self::Stmt> {
        self.processor.double(number)
    }

    fn boolean(&self, boolean: &bool) -> Result<Self::Stmt> {
        self.processor.boolean(boolean)
    }

    fn string(&self, string: &str) -> Result<Self::Stmt> {
        self.processor.string(string)
    }

    fn array(&self, values: Vec<Self::Stmt>) -> Result<Self::Stmt> {
        self.processor.array(values)
    }
}
