
#[derive(PartialEq, Debug, Clone)]
pub struct BuiltInFunction {
    pub name: String,
    pub param: TemplateString,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TemplateString {
    Variable(String),
    BuiltInFunction(Box<BuiltInFunction>),
}