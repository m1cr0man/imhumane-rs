pub mod http;
pub mod service;

#[cfg(feature = "cli")]
mod cli;

fn main() {
    #[cfg(not(feature = "cli"))]
    panic!("cli feature is not enabled");
    #[cfg(feature = "cli")]
    cli::main()
}
