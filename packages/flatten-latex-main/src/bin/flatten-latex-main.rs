//! `flatten-latex-main` binary entry point. Library lives in
//! [`flatten_latex_main`].

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use cli_common::{BaseArgs, install_json_panic_hook, parse_args_or_exit, run_no_stdout_command};
use flatten_latex_main::{FlattenOptions, flatten};

const COMMAND_NAME: &str = "flatten-latex-main";

#[derive(Parser)]
#[command(
    name = "flatten-latex-main",
    version,
    about = "Flatten a paper's main.tex into a single self-contained file"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,

    /// The paper's root TeX file (the flatten entry point). Opened directly;
    /// it need not appear in the `--file` list.
    #[arg(long, value_name = "FILE")]
    main: PathBuf,

    /// Output file path for the flattened document.
    #[arg(long, value_name = "FILE")]
    output: PathBuf,

    /// A file that may be inlined by an `\input{}` / `\subfile{}` /
    /// `\addbibresource{}` reference. Pass once per allowed file. The
    /// flattener does no directory lookup: a reference resolves only by
    /// matching its trailing path components against this fixed list, so a
    /// reference to anything not listed is refused.
    #[arg(long = "file", value_name = "FILE", required = true)]
    files: Vec<PathBuf>,

    /// Fail when a referenced bibliography file is not on the `--file` list
    /// instead of emitting a warning comment. Release builds should set this.
    #[arg(long)]
    strict_bib: bool,
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);
    let args = parse_args_or_exit::<Args>();

    run_no_stdout_command(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        flatten(
            &args.main,
            &args.files,
            &args.output,
            &FlattenOptions {
                strict_bibliography: args.strict_bib,
            },
        )?;
        tracing::info!(output = %args.output.display(), "flattened paper written");
        anyhow::Ok(())
    })
}
