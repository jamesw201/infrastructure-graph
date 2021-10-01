use crate::structs::attributes::{ AttributeType };
use crate::relationship_finders::tf_block_query::tf_block_query::JmespathExpression;

// TODO: This should return a QueryResult type and not AttributeType
// enum QueryResult {
//     AttributeList(Vec<Attribute>),
//     ValueList(Vec<AttributeType>),
//     Attribute(Attribute),
//     Value(AttributeType),
// }
pub trait Queryable {
    fn query(&self, expression: &JmespathExpression) -> Vec<AttributeType>;
}
