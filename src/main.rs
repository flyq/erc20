//! Substrate Node Template CLI library.

#![warn(missing_docs)]		// 编译器的 Lint 设置。缺乏文档时，在编译时打出警告。参考：http://llever.com/rustc-zh/print.html。
#![warn(unused_extern_crates)]		// 同上。存在未使用的外部 crate 时，在编译时打出警告。

mod chain_spec;		// 使用 chain_spec mod
mod service;
mod cli;

pub use substrate_cli::{VersionInfo, IntoExit, error};

fn run() -> cli::error::Result<()> {
	let version = VersionInfo {
		name: "Substrate Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "erc20",
		author: "flyq",
		description: "erc20",
		support_url: "support.anonymous.an",
	};
	cli::run(::std::env::args(), cli::Exit, version)
}

error_chain::quick_main!(run);
