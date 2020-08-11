use crate::structs::attributes::{ Attribute, AttributeType };
use crate::relationship_finders::tf_block_query::tf_block_query::JmespathExpression;

pub trait Queryable {
    fn query(&self, expression: JmespathExpression) -> Option<AttributeType>;
}
