use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    New { name: String },
    Make(MakeArgs),
}

#[derive(Args)]
pub struct MakeArgs {
    pub name: String,

    #[arg(short = 'm', long)]
    pub model: bool,

    #[arg(short = 'g', long)]
    pub migration: bool,

    #[arg(short = 'c', long)]
    pub controller: bool,

    #[arg(short = 'r', long)]
    pub repository: bool,
}
