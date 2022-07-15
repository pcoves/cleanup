use clap::CommandFactory;
use clap_complete::{
    generate_to,
    shells::{Bash, Zsh},
};
use std::env;
use std::io::Error;

include!("./src/options.rs");

fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-changed=src/options.rs");

    if let Ok(directory) = env::var("CARGO_MANIFEST_DIR").as_ref() {
        let command = &mut Options::command();
        let name = &command.get_name().to_string();

        println!(
            "cargo:info=Generated {:?}",
            generate_to(Bash, command, name, directory)?
        );

        println!(
            "cargo:info=Generated {:?}",
            generate_to(Zsh, command, name, directory)?
        );
    }
    Ok(())
}
