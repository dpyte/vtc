use crate::types;
use crate::memory;
use crate::types::{Types, ValType};

///
/// Container Values: Primary structure responsible for storing misc. data.
/// * `id`: used for container identification
/// * `v_type`: type of value
/// * `var_type`: type of value stored/ to be stored
/// * `intern_visibility`(True | False): Internal visibility lets the system know whether this value was passed
///     down from the serializer or system generated. The ON state defines that this field is persistent and should
///     not be changed, otherwise stated.
///     * True: Internal
///     * False: Serializer
///
struct ConValues {
	id: String,
	v_type: Types,
	var_type: ValType,
	intern_visibility: bool,
}

///
/// Attributes for the container
///
struct Attributes {

}

impl Attributes {
	pub fn new() -> Self {
		Self {  }
	}
}

///
/// Container acts as an intermediary between the data and the memory. This structure contains:
/// * c_name: container name
/// * c_values: vector of container values
/// * attr: attributes/ policy of the container
///
pub struct Container {
	c_name: String,
	c_values: Vec<ConValues>,
	attr: Attributes,
}

impl Container {
	/// Creates a new empty container
	pub fn new() -> Self {
		let c_name = "".to_string();
		let c_values = vec![];
		let attr = Attributes::new();

		Self { c_name, c_values, attr }
	}
}


