#![deny(clippy::dbg_macro)]

use std::{
    fs,
    io::{Read, Write},
    panic,
    path::{Path, PathBuf},
    process::{self, exit, Stdio},
    time::Instant,
};

use anyhow::Context;
use clap::Parser;
use console::Style;
use glob::{glob, GlobError};
use leptosfmt_formatter::{format_file_source, FormatterSettings};
use rayon::{iter::ParallelIterator, prelude::IntoParallelIterator};
use similar::{ChangeTag, TextDiff};

/// A formatter for Leptos RSX sytnax
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A space separated list of file, directory or glob
    #[arg(required_unless_present = "stdin")]
    input_patterns: Option<Vec<String>>,

    /// Maximum width of each line
    #[arg(short, long)]
    max_width: Option<usize>,

    /// Number of spaces per tab
    #[arg(short, long)]
    tab_spaces: Option<usize>,

    /// Configuration file
    #[arg(short, long)]
    config_file: Option<PathBuf>,

    /// Format stdin and write to stdout
    #[arg(short, long, default_value = "false")]
    stdin: bool,

    /// Format with rustfmt after formatting with leptosfmt (requires stdin)
    #[arg(short, long, default_value = "false", requires = "stdin")]
    rustfmt: bool,

    /// Override formatted macro names
    #[arg(long, num_args=1.., value_delimiter= ' ')]
    override_macro_names: Option<Vec<String>>,

    /// Format attributes with tailwind (by default it tries to format any attribute with the key 'class', this is configurable with --tailwind-attr-names)
    #[arg(short, long, default_value = "false")]
    experimental_tailwind: bool,

    /// Override attributes to be formatted with tailwind
    #[arg(long, num_args=1.., value_delimiter= ' ', requires = "experimental_tailwind", default_value = "class")]
    tailwind_attr_names: Vec<String>,

    #[arg(
        short,
        long,
        default_value = "false",
        default_value_if("stdin", "true", "true")
    )]
    quiet: bool,

    /// Check if the file is correctly formatted. Exit with code 1 if not.
    #[arg(long, default_value = "false")]
    check: bool,
}

fn check_if_diff(path: Option<&PathBuf>, original: &str, formatted: &str, quiet: bool) -> bool {
    if original != formatted {
        if !quiet {
            eprintln!(
                "❌ {} is not correctly formatted. See the difference below:\n",
                path.map(|p| p.display().to_string())
                    .unwrap_or("<stdin>".to_string())
            );

            let diff = TextDiff::from_lines(original, formatted);
            for change in diff.iter_all_changes() {
                let (sign, style) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new()),
                };
                eprint!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
            }
        }

        true
    } else {
        false
    }
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
        match format_stdin(settings) {
            Ok(FormatOutput {
                original,
                mut formatted,
            }) => {
                if args.rustfmt {
                    formatted = run_rustfmt(&formatted).unwrap_or(formatted);
                }

                if args.check && check_if_diff(None, &original, &formatted, true) {
                    exit(1)
                } else {
                    print!("{formatted}")
                }
            }
            Err(err) => {
                eprintln!("{err}");
                exit(1)
            }
        }
        return;
    }

    if args.rustfmt {
        // TODO: didn't dive into this yet, but `requires` clap attribute doesn't seem to work
        eprintln!("❌ --rustfmt requires --stdin");
        exit(1);
    }

    let print_err = |path: &Path, err| {
        println!("❌ {}", path.display());
        eprintln!("\t\t{}", err);
    };

    let input_patterns = args.input_patterns.unwrap();
    let file_paths: Vec<_> = get_file_paths(input_patterns).unwrap();

    let total_files = file_paths.len();
    let start_formatting = Instant::now();

    let format_results = file_paths
        .into_par_iter()
        .map(|path| (path.clone(), format_file(&path, &settings, !args.check)))
        .collect::<Vec<_>>();

    let mut check_failed = false;
    for (path, result) in format_results {
        match result {
            Ok(r) => {
                if args.check && check_if_diff(Some(&path), &r.original, &r.formatted, quiet) {
                    check_failed = true;
                }

                if !quiet {
                    println!("✅ {}", path.display())
                }
            }
            Err(err) => print_err(&path, err.to_string()),
        }
    }

    let end_formatting = Instant::now();
    if !quiet {
        println!(
            "ℹ️ {} {} files in {} ms",
            if args.check { "Checked" } else { "Formatted" },
            total_files,
            (end_formatting - start_formatting).as_millis()
        )
    }

    if check_failed {
        eprintln!("❌ Some files are not correctly formatted, see the diff above");
        exit(1);
    }
}

fn get_file_paths(input_patterns: Vec<String>) -> Result<Vec<PathBuf>, GlobError> {
    input_patterns
        .into_iter()
        .flat_map(|input_pattern| {
            let is_dir = fs::metadata(&input_pattern)
                .map(|meta| meta.is_dir())
                .unwrap_or(false);
            let glob_pattern = if is_dir {
                format!("{}/**/*.rs", &input_pattern)
            } else {
                input_pattern
            };
            glob(&glob_pattern)
                .expect("failed to read glob pattern")
                .collect::<Vec<_>>()
        })
        .collect()
}

struct FormatOutput {
    original: String,
    formatted: String,
}

fn format_stdin(settings: FormatterSettings) -> anyhow::Result<FormatOutput> {
    let mut stdin = String::new();
    let _ = std::io::stdin().read_to_string(&mut stdin);

    let formatted = panic::catch_unwind(|| format_file_source(&stdin, &settings))
        .map_err(|e| anyhow::anyhow!(e.downcast::<String>().unwrap()))??;

    Ok(FormatOutput {
        original: stdin,
        formatted,
    })
}

fn format_file(
    file: &PathBuf,
    settings: &FormatterSettings,
    write_result: bool,
) -> anyhow::Result<FormatOutput> {
    let file_source = std::fs::read_to_string(file)?;
    let formatted = panic::catch_unwind(|| format_file_source(&file_source, settings))
        .map_err(|e| anyhow::anyhow!(e.downcast::<String>().unwrap()))??;

    if write_result && file_source != formatted {
        fs::write(file, &formatted)?;
    }

    Ok(FormatOutput {
        original: file_source,
        formatted,
    })
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

    if let Some(macro_names) = args.override_macro_names.to_owned() {
        settings.macro_names = macro_names;
    }

    if args.experimental_tailwind {
        settings.attr_values = args
            .tailwind_attr_names
            .iter()
            .map(|attr_name| {
                (
                    attr_name.to_owned(),
                    leptosfmt_formatter::ExpressionFormatter::Tailwind,
                )
            })
            .collect();
    }
    Ok(settings)
}

fn load_config(path: &PathBuf) -> anyhow::Result<FormatterSettings> {
    let config = fs::read_to_string(path).context("could not read config file")?;
    let settings: FormatterSettings =
        toml::from_str(&config).context("could not parse config file")?;

    Ok(settings)
}

fn run_rustfmt(source: &str) -> Option<String> {
    let mut child = process::Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run rustfmt");

    child
        .stdin
        .as_mut()
        .expect("failed to open stdin")
        .write_all(source.as_bytes())
        .expect("failed to write to stdin");

    let output = child.wait_with_output().expect("failed to read stdout");

    if output.status.success() {
        Some(String::from_utf8(output.stdout).expect("stdout is not valid utf8"))
    } else {
        None
    }
}
