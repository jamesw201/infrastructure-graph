/// The Builder trait is for Structs that need to be created from an input string. 
/// I.e. 
/// - build a resource from a json string
/// - build a resource from a terraform string
pub trait Builder<T> {
    // fn build_from_json_string(&self, input_string: &str) -> Result<T, Error>;
    fn build_from_json_string(&self, input_string: &str) -> T;
}
