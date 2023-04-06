use std::{
    fs, panic,
    path::{Path, PathBuf},
    time::Instant,
};

use clap::Parser;
use glob::glob;
use leptosfmt_formatter::{format_file, AttributeValueBraceStyle, FormatterSettings};
use rayon::{iter::ParallelIterator, prelude::IntoParallelIterator};

/// A formatter for Leptos RSX sytnax
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A file, directory or glob
    input_pattern: String,

    // Maximum width of each line
    #[arg(short, long, default_value_t = 100)]
    max_width: usize,

    // Number of spaces per tab
    #[arg(short, long, default_value_t = 4)]
    tab_spaces: usize,

    /// Whether or not to update the files
    #[arg(short, long, default_value_t = false)]
    check: bool,
}

fn main() {
    let args = Args::parse();

    let settings = FormatterSettings {
        max_width: args.max_width,
        tab_spaces: args.tab_spaces,
        attr_value_brace_style: AttributeValueBraceStyle::WhenRequired,
        allow_changes: !args.check,
    };

    let is_dir = fs::metadata(&args.input_pattern)
        .map(|meta| meta.is_dir())
        .unwrap_or(false);

    let glob_pattern = if is_dir {
        format!("{}/**/*.rs", &args.input_pattern)
    } else {
        args.input_pattern
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
            if !settings.allow_changes {
                std::process::exit(1);
            }
        };

        match result {
            Ok(path) => match format_glob_result(&path, settings) {
                Ok(_) => println!("✅ {}", path.display()),
                Err(err) => print_err(&path, &err.to_string()),
            },
            Err(err) => print_err(err.path(), &err.error().to_string()),
        };
    });
    let end_formatting = Instant::now();
    println!(
        "Formatted {} files in {} ms",
        total_files,
        (end_formatting - start_formatting).as_millis()
    )
}

fn format_glob_result(file: &PathBuf, settings: FormatterSettings) -> anyhow::Result<()> {
    let formatted = panic::catch_unwind(|| format_file(file, settings))
        .map_err(|e| anyhow::anyhow!(e.downcast::<String>().unwrap()))??;
    fs::write(file, formatted)?;
    Ok(())
}
