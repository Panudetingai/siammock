mod app;
mod cli;
mod compiler;
mod config;
mod data;
mod handlers;
mod response;
mod router;
mod validation;

use clap::Parser;
use cli::{Cli, Commands, ValidateArgs};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Validate(args)) => {
            if let Err(code) = run_validate(args) {
                std::process::exit(code);
            }
        }
        Some(Commands::Start(args)) => app::run(args).await,
        None => app::run(cli.start_args()).await,
    }
}

fn run_validate(args: ValidateArgs) -> Result<(), i32> {
    let combined = if args.stdin {
        validate_stdin(&args)?
    } else {
        let config = args.config.as_deref().unwrap_or("mock");
        validate_files(config)?
    };

    if args.json {
        let output = serde_json::to_string_pretty(&combined).map_err(|err| {
            eprintln!("failed to serialize diagnostics: {err}");
            1
        })?;
        println!("{output}");
    } else {
        print_human_diagnostics(&combined);
    }

    if combined.valid {
        Ok(())
    } else {
        Err(1)
    }
}

fn validate_stdin(args: &ValidateArgs) -> Result<compiler::CompileResult, i32> {
    use std::io::Read;

    let mut source = String::new();
    std::io::stdin()
        .read_to_string(&mut source)
        .map_err(|err| {
            eprintln!("failed to read stdin: {err}");
            1
        })?;

    let label = args
        .file
        .as_deref()
        .unwrap_or("<stdin>");

    Ok(compiler::validate_with_path(&source, label))
}

fn validate_files(config: &str) -> Result<compiler::CompileResult, i32> {
    let files = config::loader::resolve_config_files(config).map_err(|err| {
        eprintln!("{err}");
        1
    })?;

    let mut combined = compiler::CompileResult {
        valid: true,
        diagnostics: Vec::new(),
    };

    for file in &files {
        let source = std::fs::read_to_string(file).map_err(|err| {
            eprintln!("failed to read {}: {err}", file.display());
            1
        })?;

        let result = compiler::validate_with_path(&source, &file.display().to_string());
        combined = combined.merge(result);
    }

    Ok(combined)
}

fn print_human_diagnostics(result: &compiler::CompileResult) {
    if result.valid {
        println!("Config is valid.");
        return;
    }

    for diagnostic in &result.diagnostics {
        let level = match diagnostic.severity {
            compiler::Severity::Error => "error",
            compiler::Severity::Warning => "warning",
            compiler::Severity::Info => "info",
        };

        eprintln!(
            "[{level}] {}:{}:{} {} — {}",
            diagnostic.path, diagnostic.line, diagnostic.column, diagnostic.code, diagnostic.message
        );

        if let Some(hint) = &diagnostic.hint {
            eprintln!("  hint: {hint}");
        }
    }
}
