//! This is the snx CLI and is used to create new snx projects or generate
//! scaffolding.
mod cli;

use std::{fs, process};

use anyhow::bail;
use clap::Parser;
use nanocolor::Colorize;
use nanospinner::Spinner;

use cli::{Cli, Command, MakeArgs};
use pluralizer::pluralize;

const STUB_CARGO_TOML: &str = include_str!("../stubs/new/Cargo.toml");
const STUB_MAIN_RS: &str = include_str!("../stubs/new/main.rs");
const STUB_GITIGNORE: &str = include_str!("../stubs/new/.gitignore");
const STUB_MODEL: &str = include_str!("../stubs/make/model.rs");
const STUB_MIGRATION: &str = include_str!("../stubs/make/migration.sql");
const STUB_CONTROLLER: &str = include_str!("../stubs/make/controller.rs");
const STUB_REPOSITORY: &str = include_str!("../stubs/make/repository.rs");

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::New { name } => new(&name)?,
        Command::Make(args) => make(args)?,
    }

    Ok(())
}

/// Create a new project
///
/// Creates a project directory, copies stub files and runs cargo fetch.
fn new(name: &str) -> anyhow::Result<()> {
    let display_name = if name == "." {
        std::env::current_dir()?
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    } else {
        name.to_string()
    };

    let dirs = ["src"];
    for dir in dirs {
        fs::create_dir_all(format!("{name}/{dir}"))?;
    }
    println!("{} directories created", "✔".green().bold());

    fs::write(
        format!("{name}/Cargo.toml"),
        STUB_CARGO_TOML.replace("{{name}}", &display_name),
    )?;
    fs::write(format!("{name}/src/main.rs"), STUB_MAIN_RS)?;
    fs::write(format!("{name}/.gitignore"), STUB_GITIGNORE)?;
    println!("{} stubs copied", "✔".green().bold());

    let handle = Spinner::new("fetching dependencies...").start();
    let output = process::Command::new("cargo")
        .arg("fetch")
        .current_dir(name)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::piped())
        .output()?;

    if output.status.success() {
        handle.success_with("dependencies fetched");
    } else {
        handle.fail();
        bail!(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let cd_line = if name != "." {
        format!("  {}\n", format!("cd {name}").bright_black().italic())
    } else {
        String::new()
    };

    println!(
        "\n {} {display_name} is ready\n\n{}  {}\n",
        " snx ".black().on_bright_white().bold(),
        cd_line,
        "cargo run".bright_black().italic()
    );

    Ok(())
}

/// Scaffold components
///
/// Accepts any form of name (singular, plural, capitalized, etc.) and unifies
/// it. Creates directories and copies stub files
pub fn make(args: MakeArgs) -> anyhow::Result<()> {
    if args.model {
        let singular_lowercase = pluralize(&args.name, 1, false);
        let capitalized = capitalize(&singular_lowercase);
        fs::create_dir_all("src/models")?;
        let path = format!("src/models/{singular_lowercase}.rs");
        fs::write(&path, STUB_MODEL.replace("{{name}}", &capitalized))?;
        println!(
            "{} model created {}",
            "✔".green().bold(),
            path.bright_black().italic()
        );
    }

    if args.migration {
        let plural_lowercase = pluralize(&args.name, 2, false).to_lowercase();
        fs::create_dir_all("src/migrations")?;
        let path = format!("src/migrations/create_{plural_lowercase}_table.sql");
        fs::write(&path, STUB_MIGRATION.replace("{{name}}", &plural_lowercase))?;
        println!(
            "{} migration created {}",
            "✔".green().bold(),
            path.bright_black().italic()
        );
    }

    if args.controller {
        let plural_lowercase = pluralize(&args.name, 2, false).to_lowercase();
        fs::create_dir_all("src/controllers")?;
        let path = format!("src/controllers/{plural_lowercase}.rs");
        fs::write(&path, STUB_CONTROLLER)?;
        println!(
            "{} controller created {}",
            "✔".green().bold(),
            path.bright_black().italic()
        );
    }

    if args.repository {
        let singular_lowercase = pluralize(&args.name, 1, false).to_lowercase();
        let capitalized = capitalize(&singular_lowercase);
        fs::create_dir_all("src/repositories")?;
        let path = format!("src/repositories/{singular_lowercase}.rs");
        fs::write(&path, STUB_REPOSITORY.replace("{{name}}", &capitalized))?;
        println!(
            "{} repository created {}",
            "✔".green().bold(),
            path.bright_black().italic()
        );
    }

    Ok(())
}

fn capitalize(str: &str) -> String {
    let mut chars = str.chars();
    let first = chars.next().unwrap();

    first.to_uppercase().to_string() + &chars.collect::<String>()
}
