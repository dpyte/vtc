/// Types: Type of value
#[derive(Debug)]
pub enum Types {
	I8,
	I16,
	I32,
	I64,
	U8,
	U16,
	U32,
	U64,
	Float8,
	Float16,
	Float32,
	Float64,
	Float128,
	Char,
	Str,
}

///
/// ValType: Annotates type of value contained in ConValues:
///  * Ptr: Pointer (String): Stores container id
///  * Ref: Reference (String): Stores container id
///  * Value
///
#[derive(Debug)]
pub enum ValType {
	Ptr,
	Ref,
	Value,
}
