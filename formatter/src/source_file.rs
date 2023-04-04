use std::{io, ops::Range};

use crop::Rope;
use syn::{parse_str, spanned::Spanned, Expr, Macro, MacroDelimiter};
use thiserror::Error;

use crate::{
    collect::{collect_macros_in_expr, collect_macros_in_file},
    formatter::{format_macro, FormatterSettings},
};

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("could not read file")]
    IoError(#[from] io::Error),
    #[error("could not parse file")]
    ParseError(#[from] syn::Error),
    #[error("found files that needed formatting")]
    IncorrectFormatError,
}

#[derive(Debug)]
struct TextEdit {
    range: Range<usize>,
    new_text: String,
}

pub(crate) fn format_file_source(
    source: &str,
    settings: FormatterSettings,
) -> Result<String, FormatError> {
    let ast = syn::parse_file(source)?;
    let macros = collect_macros_in_file(&ast);
    format_source(source, macros, settings)
}

pub(crate) fn format_expr_source(
    source: &str,
    settings: FormatterSettings,
) -> Result<String, FormatError> {
    let ast: Expr = parse_str(source)?;
    let macros = collect_macros_in_expr(&ast);
    format_source(source, macros, settings)
}

fn format_source<'a>(
    source: &'a str,
    macros: Vec<&'a Macro>,
    settings: FormatterSettings,
) -> Result<String, FormatError> {
    let mut source: Rope = source.parse().unwrap();
    let mut edits = Vec::new();

    for mac in macros {
        let start = mac.path.span().start();
        let end = match mac.delimiter {
            MacroDelimiter::Paren(delim) => delim.span.end(),
            MacroDelimiter::Brace(delim) => delim.span.end(),
            MacroDelimiter::Bracket(delim) => delim.span.end(),
        };

        let start_byte = source.byte_of_line(start.line - 1) + start.column;
        let end_byte = source.byte_of_line(end.line - 1) + end.column;
        let new_text = format_macro(mac, settings);

        edits.push(TextEdit {
            range: start_byte..end_byte,
            new_text,
        });
    }

    let mut last_offset: isize = 0;
    for edit in edits {
        let start = edit.range.start;
        let end = edit.range.end;
        let new_text = edit.new_text;

        source.replace(
            (start as isize + last_offset) as usize..(end as isize + last_offset) as usize,
            &new_text,
        );
        last_offset += new_text.len() as isize - (end as isize - start as isize);
    }

    Ok(source.to_string())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn it_works() {
        let source = indoc! {r#"
            fn main() {
                view! {   cx ,  <div>  <span>"hello"</span></div>  };
            }
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        fn main() {
            view! { cx,
                <div>
                    <span>"hello"</span>
                </div>
            };
        }
        "###);
    }

    #[test]
    fn nested() {
        let source = indoc! {r#"
            fn main() {
                view! {   cx ,  <div>  <span>{
                        let a = 12;

                        view! { cx,

                                         <span>{a}</span>
                        }
                }</span></div>  };
            }
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        fn main() {
            view! { cx,
                <div>
                    <span>
                        {
                            let a = 12;
                            view! { cx, <span>{a}</span> }
                        }
                    </span>
                </div>
            };
        }
        "###);
    }

    #[test]
    fn multiple() {
        let source = indoc! {r#"
            fn main() {
                view! {   cx ,  <div>  <span>"hello"</span></div>  };
                view! {   cx ,  <div>  <span>"hello"</span></div>  };
            }
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        fn main() {
            view! { cx,
                <div>
                    <span>"hello"</span>
                </div>
            };
            view! { cx,
                <div>
                    <span>"hello"</span>
                </div>
            };
        }
        "###);
    }
}
