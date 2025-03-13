use std::env;
use std::sync::Once;

/// Print citation information (only prints once)
pub fn print_citation() {
	static PRINT_CITATION_ONCE: Once = Once::new();
	PRINT_CITATION_ONCE.call_once(|| {
		eprintln!("Citation: Neil R Voss, et al. J Mol Biol. v360 (4): 2006, pp. 893-906.");
		eprintln!("DOI: http://dx.doi.org/10.1016/j.jmb.2006.05.023");
		eprintln!("E-mail: M Gerstein <mark.gerstein@yale.edu> or NR Voss <vossman77@yahoo.com>\n");
	});
}

/// Print compilation information (only prints once)
pub fn print_compile_info() {
	static PRINT_COMPILE_ONCE: Once = Once::new();
	PRINT_COMPILE_ONCE.call_once(|| {
		// Get the executable name
		let program_name = env::current_exe()
		.ok()
		.as_ref()
		.and_then(|path| path.file_name()) // Extract filename
		.and_then(|name| name.to_str()) // Convert to &str
		.unwrap_or("Unknown Program") // Fallback

		.to_string();

		eprintln!("Program: {}", program_name);
		eprintln!(
			"Compiled on: {} at {}",
			env!("COMPILE_DATE"),
					 env!("COMPILE_TIME")
		);
		eprintln!("Rust version: {}", env!("CARGO_PKG_VERSION"));
	});
}
