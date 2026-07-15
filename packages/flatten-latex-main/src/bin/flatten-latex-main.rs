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

    /// Directory containing the paper's main TeX file. `\input{}`,
    /// `\subfile{}`, and `\addbibresource{}` paths resolve relative to it.
    #[arg(long, value_name = "DIR")]
    paper_dir: PathBuf,

    /// Output file path for the flattened document.
    #[arg(long, value_name = "FILE")]
    output: PathBuf,

    /// Additional directory searched (in order, after `--paper-dir`) when a
    /// referenced `\input{}` / `\subfile{}` / `\addbibresource{}` file is not
    /// found next to the paper's sources. May be passed multiple times; used
    /// for build-staged shared macros.
    #[arg(long = "include-dir", value_name = "DIR")]
    include_dirs: Vec<PathBuf>,

    /// Main TeX file to flatten. Relative paths resolve against `--paper-dir`.
    #[arg(long, value_name = "FILE", default_value = "main.tex")]
    main: PathBuf,

    /// Fail when a referenced bibliography file is missing instead of
    /// emitting a warning comment. Release builds should set this.
    #[arg(long)]
    strict_bib: bool,
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);
    let args = parse_args_or_exit::<Args>();

    run_no_stdout_command(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        let main_file = if args.main.is_absolute() {
            args.main.clone()
        } else {
            args.paper_dir.join(&args.main)
        };
        flatten(
            &args.paper_dir,
            &main_file,
            &args.output,
            &args.include_dirs,
            &FlattenOptions {
                strict_bibliography: args.strict_bib,
            },
        )?;
        tracing::info!(output = %args.output.display(), "flattened paper written");
        anyhow::Ok(())
    })
}
