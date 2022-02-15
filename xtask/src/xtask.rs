//! Build/test operations for the `exhaust` workspace.
//!
//! This is an instance of the `cargo-xtask` pattern as described by
//! <https://github.com/matklad/cargo-xtask>.

use xaction::Cmd;

#[derive(Debug, clap::Parser)]
struct XtaskArgs {
    #[clap(subcommand)]
    command: XtaskCommand,
}

#[derive(Debug, clap::Subcommand)]
enum XtaskCommand {
    /// Run tests in all feature/target combinations we want to exercise.
    /// Also builds documentation.
    Test,
}

fn main() -> Result<(), xaction::Error> {
    let XtaskArgs { command } = <XtaskArgs as clap::Parser>::parse();

    match command {
        XtaskCommand::Test => {
            exhaustive_test()?;
            cargo().arg("doc").run()?;
        }
    }
    Ok(())
}

fn exhaustive_test() -> Result<(), xaction::Error> {
    // All defaults
    test_under_conditions([])?;

    // Try with alloc but not std
    test_under_conditions(["--no-default-features"])?;

    // Try with alloc but not std
    test_under_conditions(["--no-default-features", "--features=alloc"])?;

    // A no_std target, so that any std deps will definitely fail to compile
    cargo()
        .args([
            "check",
            "--no-default-features",
            "--target=thumbv6m-none-eabi",
        ])
        .run()?;

    Ok(())
}

fn test_under_conditions<'a, A: AsRef<[&'a str]>>(flags: A) -> Result<(), xaction::Error> {
    cargo().arg("test").args(flags.as_ref()).run()?;
    Ok(())
}

/// Start a [`Cmd`] with the cargo command we should use.
fn cargo() -> Cmd {
    Cmd::new(std::env::var("CARGO").expect("CARGO environment variable not set"))
}
