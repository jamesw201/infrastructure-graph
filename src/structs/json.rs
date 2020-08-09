
#[derive(PartialEq, Debug, Clone)]
pub enum JsonValue {
  Str(String),
  Boolean(bool),
  Null(String),
  Num(f64),
  Array(Vec<JsonValue>),
  Object(Vec<(String, JsonValue)>),
}