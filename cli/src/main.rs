use std::{
    fs,
    io::Read,
    panic,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::Context;
use clap::{Parser, builder::ArgPredicate};
use glob::glob;
use leptosfmt_formatter::{format_file,FormatterSettings, format_file_source};
use rayon::{iter::ParallelIterator, prelude::IntoParallelIterator};

/// A formatter for Leptos RSX sytnax
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A file, directory or glob
    #[arg(required_unless_present = "stdin")]
    input_pattern: Option<String>,

    // Maximum width of each line
    #[arg(short, long)]
    max_width: Option<usize>,

    // Number of spaces per tab
    #[arg(short, long)]
    tab_spaces: Option<usize>,

    // Config file
    #[arg(short, long)]
    config_file: Option<PathBuf>,

    #[arg(short, long, default_value = "false")]
    stdin: bool,

    #[arg(short, long, default_value = "false", default_value_if("stdin", ArgPredicate::IsPresent, "true"))]
    quiet: bool,
}

fn main() {
    let args = Args::parse();

    let settings = create_settings(&args).unwrap();
    let quiet = args.quiet;

    // Print settings
    if !quiet {
        println!("{}", toml::to_string_pretty(&settings).unwrap());
    }

    if args.stdin {
        match format_stdin_result(settings) {
            Ok(_) => {}
            Err(err) => eprintln!("{}", err),
        };
    } else {
        let input_pattern = args.input_pattern.unwrap();
        let is_dir = fs::metadata(&input_pattern)
            .map(|meta| meta.is_dir())
            .unwrap_or(false);

        let glob_pattern = if is_dir {
            format!("{}/**/*.rs", &input_pattern)
        } else {
            input_pattern
        };

        let file_paths: Vec<_> = glob(&glob_pattern)
            .expect("failed to read glob pattern")
            .collect();

        let total_files = file_paths.len();
        let start_formatting = Instant::now();
        file_paths.into_par_iter().for_each(|result| {
            let print_err = |path: &Path, err| {
                println!("❌ {}", path.display());
                eprintln!("\t\t{}", err);
            };

            match result {
                Ok(path) => match format_glob_result(&path, settings) {
                    Ok(_) => {
                        if !quiet {
                            println!("✅ {}", path.display())
                        }
                    }
                    Err(err) => print_err(&path, &err.to_string()),
                },
                Err(err) => print_err(err.path(), &err.error().to_string()),
            };
        });
        let end_formatting = Instant::now();
        if !quiet {
            println!(
                "Formatted {} files in {} ms",
                total_files,
                (end_formatting - start_formatting).as_millis()
            )
        }
    }
}

fn format_stdin_result(settings: FormatterSettings) -> anyhow::Result<()> {
    let mut stdin = String::new();
    let _ = std::io::stdin().read_to_string(&mut stdin);

    let formatted = panic::catch_unwind(|| format_file_source(&stdin, settings))
        .map_err(|e| anyhow::anyhow!(e.downcast::<String>().unwrap()))??;

    print!("{}", formatted);

    Ok(())
}

fn format_glob_result(file: &PathBuf, settings: FormatterSettings) -> anyhow::Result<()> {
    let formatted = panic::catch_unwind(|| format_file(file, settings))
        .map_err(|e| anyhow::anyhow!(e.downcast::<String>().unwrap()))??;
    fs::write(file, formatted)?;
    Ok(())
}

fn create_settings(args: &Args) -> anyhow::Result<FormatterSettings> {
    let mut settings = args
        .config_file
        .as_ref()
        .map(|path| {
            load_config(path)
                .with_context(|| format!("failed to load config file: {}", path.display()))
        })
        .unwrap_or_else(|| {
            let default_config: PathBuf = "leptosfmt.toml".into();
            if default_config.exists() {
                load_config(&default_config).with_context(|| {
                    format!("failed to load config file: {}", default_config.display())
                })
            } else {
                Ok(FormatterSettings::default())
            }
        })?;

    if let Some(max_width) = args.max_width {
        settings.max_width = max_width;
    }

    if let Some(tab_spaces) = args.tab_spaces {
        settings.tab_spaces = tab_spaces;
    }
    Ok(settings)
}

fn load_config(path: &PathBuf) -> anyhow::Result<FormatterSettings> {
    let config = fs::read_to_string(path).context("could not read config file")?;
    let settings: FormatterSettings =
        toml::from_str(&config).context("could not parse config file")?;

    Ok(settings)
}
