use super::*;

#[derive(Debug, Serialize)]
pub struct RpOptionDecl {
    pub name: String,
    pub values: Vec<RpLoc<RpValue>>,
}
