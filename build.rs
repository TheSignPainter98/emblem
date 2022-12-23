mod args {
    #![allow(dead_code)]
    include!("src/args.rs");
}

use args::RawArgs;
use clap::CommandFactory;
use clap_complete::shells::Shell;
use clap_mangen::Man;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    complgen()?;
    mangen()?;
    parsergen()?;
    Ok(())
}

fn mangen() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/args.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_file = Path::new(&out_dir).join("em.1");

    let mut file = File::create(dest_file)?;
    Man::new(RawArgs::command()).render(&mut file)?;
    drop(file);
    Ok(())
}

fn complgen() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/args.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_dir = Path::new(&out_dir).join("completion");

    if !dest_dir.exists() {
        fs::create_dir(dest_dir.clone())?;
    }

    let shells = [
        Shell::Bash,
        Shell::Elvish,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Zsh,
    ];
    for shell in shells {
        clap_complete::generate_to(shell, &mut RawArgs::command(), "em", dest_dir.clone())?;
    }
    Ok(())
}

fn parsergen() -> Result<(), Box<dyn Error>> {
    lalrpop::Configuration::new()
        .set_in_dir("src/parser/")
        .set_out_dir(Path::new(&env::var("OUT_DIR")?).join("parser"))
        .emit_rerun_directives(true)
        .process()
}
