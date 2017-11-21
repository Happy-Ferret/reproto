use core::RpNumber;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'input> {
    Identifier(&'input str),
    TypeIdentifier(&'input str),
    PackageDocComment(Vec<&'input str>),
    DocComment(Vec<&'input str>),
    Number(RpNumber),
    LeftCurly,
    RightCurly,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    SemiColon,
    Colon,
    Equal,
    Comma,
    Dot,
    Scope,
    QuestionMark,
    RightArrow,
    CodeOpen,
    CodeClose,
    CodeContent(&'input str),
    String(String),
    // identifier-style keywords
    InterfaceKeyword,
    TypeKeyword,
    EnumKeyword,
    TupleKeyword,
    ServiceKeyword,
    UseKeyword,
    AsKeyword,
    AnyKeyword,
    FloatKeyword,
    DoubleKeyword,
    Signed32,
    Signed64,
    Unsigned32,
    Unsigned64,
    BooleanKeyword,
    StringKeyword,
    DateTimeKeyword,
    BytesKeyword,
    TrueKeyword,
    FalseKeyword,
    StreamKeyword,
    OptionKeyword,
}
