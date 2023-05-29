use std::{io, ops::Range};

use crop::Rope;
use syn::{parse_str, spanned::Spanned, Expr, MacroDelimiter};
use thiserror::Error;

use crate::{
    collect::{collect_macros_in_expr, collect_macros_in_file},
    formatter::{format_macro, FormatterSettings},
    ViewMacro,
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
    macros: Vec<ViewMacro<'a>>,
    settings: FormatterSettings,
) -> Result<String, FormatError> {
    let mut source: Rope = source.parse().unwrap();
    let mut edits = Vec::new();

    for view_mac in macros {
        let mac = view_mac.inner();
        let start = mac.path.span().start();
        let end = match mac.delimiter {
            MacroDelimiter::Paren(delim) => delim.span.end(),
            MacroDelimiter::Brace(delim) => delim.span.end(),
            MacroDelimiter::Bracket(delim) => delim.span.end(),
        };

        let start_byte = line_column_to_byte(&source, start);
        let end_byte = line_column_to_byte(&source, end);
        let new_text = format_macro(&view_mac, settings);

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

fn line_column_to_byte(source: &Rope, point: proc_macro2::LineColumn) -> usize {
    let line_byte = source.byte_of_line(point.line - 1);
    let line = source.line(point.line - 1);
    let char_byte: usize = line.chars().take(point.column).map(|c| c.len_utf8()).sum();
    line_byte + char_byte
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

    #[test]
    fn with_special_characters() {
        let source = indoc! {r#"
            fn main() {
                view! {   cx ,  <div>  <span>"hello²💣"</span></div>  }; 
            }
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        fn main() {
            view! { cx,
                <div>
                    <span>"hello²💣"</span>
                </div>
            }; 
        }
        "###);
    }

    #[test]
    fn inside_match_case() {
        let source = indoc! {r#"
            use leptos::*;

            enum ExampleEnum {
                ValueOneWithAReallyLongName,
                ValueTwoWithAReallyLongName,
            }

            #[component]
            fn Component(cx: Scope, val: ExampleEnum) -> impl IntoView {
                match val {
                    ExampleEnum::ValueOneWithAReallyLongName => 
                        view! { cx,
                                                                    <div>
                                                                        <div>"Value One"</div>
                                                                    </div>
                                                                }.into_view(cx),
                    ExampleEnum::ValueTwoWithAReallyLongName =>  view! { cx,
                                                                    <div>
                                                                        <div>"Value Two"</div>
                                                                    </div>
                                                                }.into_view(cx),
                };
            }
        "#};

        let result = format_file_source(source, Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        use leptos::*;

        enum ExampleEnum {
            ValueOneWithAReallyLongName,
            ValueTwoWithAReallyLongName,
        }

        #[component]
        fn Component(cx: Scope, val: ExampleEnum) -> impl IntoView {
            match val {
                ExampleEnum::ValueOneWithAReallyLongName => 
                    view! { cx,
                        <div>
                            <div>"Value One"</div>
                        </div>
                    }.into_view(cx),
                ExampleEnum::ValueTwoWithAReallyLongName =>  view! { cx,
                        <div>
                            <div>"Value Two"</div>
                        </div>
                    }.into_view(cx),
            };
        }
        "###);
    }
}
