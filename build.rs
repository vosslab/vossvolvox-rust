use std::process::Command;

fn main() {
	let output = Command::new("date")
	.arg("+%Y-%m-%d")
	.output()
	.expect("Failed to get date");

	let compile_date = String::from_utf8_lossy(&output.stdout).trim().to_string();

	let output = Command::new("date")
	.arg("+%H:%M:%S")
	.output()
	.expect("Failed to get time");

	let compile_time = String::from_utf8_lossy(&output.stdout).trim().to_string();

	println!("cargo:rustc-env=COMPILE_DATE={}", compile_date);
	println!("cargo:rustc-env=COMPILE_TIME={}", compile_time);
}
