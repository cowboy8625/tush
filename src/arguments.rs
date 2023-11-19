use clap::{crate_name, crate_version, Parser};

const AUTHOR: &str = "cowboy8625";

#[derive(Debug, Parser)]
#[command(
    name = crate_name!(),
    version = crate_version!(),
    author = AUTHOR,
    about = "a tool for posting issues to github",
    long_about = None
    )]
pub struct Cli {
    #[arg(long, short, default_value = ".")]
    pub path: String,
    #[arg(long, short, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub check: bool,
}
