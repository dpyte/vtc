use std::path::PathBuf;
use vtc::parser::ast::ReferenceType::External;
use vtc::runtime::runtime::Runtime;

fn main() {
    let mut rt = Runtime::new();
    rt.read_file(PathBuf::from("./tests/inherit_directive.vtc")).unwrap();
    let value = rt.get_value("test_sample", "value_2", External, vec![]).unwrap();
    println!("value: {:#}", value);
}
