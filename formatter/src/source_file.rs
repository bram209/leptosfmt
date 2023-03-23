use std::{io, ops::Range};

use crop::Rope;
use proc_macro2::LineColumn;
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
    let mut rope: Rope = source.parse().unwrap();
    let mut edits = Vec::new();

    for mac in macros {
        let start = mac.path.span().start();
        let end = match mac.delimiter {
            MacroDelimiter::Paren(delim) => delim.span.end(),
            MacroDelimiter::Brace(delim) => delim.span.end(),
            MacroDelimiter::Bracket(delim) => delim.span.end(),
        };

        let start_byte = line_column_to_byte_index(&rope, start);
        let end_byte = line_column_to_byte_index(&rope, end);
        let new_text = format_macro(Some(&source), mac, settings);

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

        rope.replace(
            (start as isize + last_offset) as usize..(end as isize + last_offset) as usize,
            &new_text,
        );
        last_offset += new_text.len() as isize - (end as isize - start as isize);
    }

    Ok(rope.to_string())
}

fn line_column_to_byte_index(rope: &Rope, line_column: LineColumn) -> usize {
    rope.byte_of_line(line_column.line - 1) + line_column.column
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
    fn with_comments() {
        let source = indoc! {r#"
            fn main() {
                view! {   cx ,  <div>  
        // This is one beautiful message
                    <span>"hello"</span> // at the end of the line
                    <div>// at the end of the line
                    <span>"hello"</span> </div>
                     <For
            // a function that returns the items we're iterating over; a signal is fine
            each= move || {errors.clone().into_iter().enumerate()}
            // a unique key for each item as a reference
             key=|(index, _error)| *index // yeah
             // double
             // comments
             multiline={
                let a = 12;
                a + 2 // nice calculation
             }
             />
                    </div>  }; 
            }
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
            fn main() {
                view! { cx,
                    <div>
                        // This is one beautiful message
                        <span>"hello"</span>
                        <For
                            // a function that returns the items we're iterating over; a signal is fine
                            each=move || { errors.clone().into_iter().enumerate() }
                            // a unique key for each item as a reference
                            key=|(index, _error)| *index
                        />
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
