#![allow(warnings)]

fn main() {
	print!("{:?}", {
		// The following gets replaced by the buffer
		"::_rspl_main_::";
	});
}
