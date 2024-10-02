use std::path::PathBuf;
use vtc::parser::ast::Accessor::Index;
use vtc::runtime::error::RuntimeError;
use vtc::runtime::runtime::Runtime;

fn main() -> Result<(), RuntimeError> {
    let mut rt = Runtime::new();
    let mut read_rt = Runtime::new();

    read_rt.load_file(PathBuf::from("./tests/read_pair.vtc"))?;
    let namespace = read_rt.to_string(
        read_rt.get_value("read", "pair", &[Index(0)])?)?;
    let variable = read_rt.to_string(
        read_rt.get_value("read", "pair", &[Index(1)])?)?;

    rt.load_file(PathBuf::from("./samples/system.vtc"))?;
    let mem_ptr = rt.get_boolean(namespace.as_str(), variable.as_str())?;
    println!("=> {:?}", mem_ptr);

    Ok(())
}
