#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

/// ANSI SGR 序列：加粗 + 绿色前景
const GREEN: &str = "\x1b[1;32m";
/// ANSI SGR 序列：复位所有属性
const RESET: &str = "\x1b[0m";

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("{GREEN}[WithColor]: Hello, Arceos!{RESET}");
}
