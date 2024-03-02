use std::{
    io::{self},
    ops::Range,
};

use crop::Rope;

use syn::spanned::Spanned;
use thiserror::Error;

use crate::{
    collect::collect_macros_in_file,
    formatter::{format_macro, FormatterSettings},
    line_column_to_byte, ViewMacro,
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

pub fn format_file_source(
    source: &str,
    settings: &FormatterSettings,
) -> Result<String, FormatError> {
    let ast = syn::parse_file(source)?;
    let rope = Rope::from(source);
    let (mut rope, macros) = collect_macros_in_file(&ast, rope, &settings.macro_names);
    format_source(&mut rope, macros, settings)
}

fn format_source(
    source: &mut Rope,
    macros: Vec<ViewMacro<'_>>,
    settings: &FormatterSettings,
) -> Result<String, FormatError> {
    let mut edits = Vec::new();

    for view_mac in macros {
        let mac = view_mac.inner();
        let start = mac.path.span().start();
        let end = mac.delimiter.span().close().end();
        let start_byte = line_column_to_byte(source, start);
        let end_byte = line_column_to_byte(source, end);
        let new_text = format_macro(&view_mac, settings, Some(source));

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

    use crate::IndentationStyle;

    use super::*;

    #[test]
    fn rustfmt_leptosfmt_indent_difference() {
        let source = indoc! {r#"
        // Valid Rust formatted code
        #[component]
        pub(crate) fn Error(cx: Scope, message: Option<String>) -> impl IntoView {
            view! { cx,
              <div>
                Example
              </div>
            }
        }
        "#};

        let result = format_file_source(
            source,
            &FormatterSettings {
                tab_spaces: 2,
                ..Default::default()
            },
        )
        .unwrap();
        insta::assert_snapshot!(result, @r###"
        // Valid Rust formatted code
        #[component]
        pub(crate) fn Error(cx: Scope, message: Option<String>) -> impl IntoView {
            view! { cx, <div>Example</div> }
        }
        "###);
    }

    #[test]
    fn it_works() {
        let source = indoc! {r#"
            fn main() {
                view! {   cx ,  <div>  <span>"hello"</span></div>  }; 
            }
        "#};

        let result = format_file_source(source, &Default::default()).unwrap();
        insta::assert_snapshot!(result, @r#"
        fn main() {
            view! { cx,
                <div>
                    <span>"hello"</span>
                </div>
            }; 
        }

        "#);
    }

    #[test]
    fn fully_qualified_macro_path() {
        let source = indoc! {r#"
            fn main() {
                leptos::view! {   cx ,  <div>  <span>"hello"</span></div>  }; 
            }
        "#};

        let result = format_file_source(source, &Default::default()).unwrap();
        insta::assert_snapshot!(result, @r#"
        fn main() {
            leptos::view! { cx,
                <div>
                    <span>"hello"</span>
                </div>
            }; 
        }

        "#);
    }

    #[test]
    fn ignore_other_macros() {
        let source = indoc! {r#"
            fn main() {
                leptos::view! {   cx ,  <div class=format!("classy")>  <span>"hello"</span></div>  }; 
            }
        "#};

        let result = format_file_source(source, &Default::default()).unwrap();
        insta::assert_snapshot!(result, @r#"
        fn main() {
            leptos::view! { cx,
                <div class=format!("classy")>
                    <span>"hello"</span>
                </div>
            }; 
        }

        "#);
    }

    #[test]
    fn fully_qualified_macro_path_overridden() {
        let source = indoc! {r#"
            fn main() {
                foo::bar::some_view! {   cx ,  <div>  <span>"hello"</span></div>  }; 
            }
        "#};

        let result = format_file_source(
            source,
            &FormatterSettings {
                macro_names: vec!["foo::bar::some_view".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        insta::assert_snapshot!(result, @r#"
        fn main() {
            foo::bar::some_view! { cx,
                <div>
                    <span>"hello"</span>
                </div>
            }; 
        }

        "#);
    }

    #[test]
    fn fully_qualified_macro_path_with_indent() {
        let source = indoc! {r#"
            fn main() {
                foo::bar::some_view! {   cx ,  <div>  <span>{
                        let a = 12;


                        foo::bar::some_view! { cx,             
                            
                                         <span>{a}</span>
                        }
                }</span></div>  };
            }
        "#};

        let result = format_file_source(
            source,
            &FormatterSettings {
                macro_names: vec!["foo::bar::some_view".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        insta::assert_snapshot!(result, @r#"
        fn main() {
            foo::bar::some_view! { cx,
                <div>
                    <span>
                        {
                            let a = 12;

                            foo::bar::some_view! { cx, <span>{a}</span> }
                        }
                    </span>
                </div>
            };
        }

        "#);
    }

    #[test]
    fn override_macro_names() {
        let source = indoc! {r#"
            fn main() {
                html! {   cx ,  <div>  <span>{
                        let a = 12;


                        html! { cx,             
                            
                                         <span>{a}</span>
                        }
                }</span></div>  };
            }
        "#};

        let result = format_file_source(
            source,
            &FormatterSettings {
                macro_names: vec!["html".to_string()],
                ..Default::default()
            },
        )
        .unwrap();
        insta::assert_snapshot!(result, @r#"
        fn main() {
            html! { cx,
                <div>
                    <span>
                        {
                            let a = 12;

                            html! { cx, <span>{a}</span> }
                        }
                    </span>
                </div>
            };
        }

        "#);
    }

    #[test]
    fn with_comments() {
        let source = indoc! {r#"
            // comment outside view macro
            fn main() {
                view! {   cx ,
                    // Top level comment
                    <div>
                        // This is one beautiful message
                    <span>"hello"</span> // at the end of the line 1
                    <div>// at the end of the line 2
             // double
             // comments
                    <span>"hello"</span> </div>
                     <For
            // a function that returns the items we're iterating over; a signal is fine
            each= move || {errors.clone().into_iter().enumerate()}
            // a unique key for each item as a reference
             key=|(index, _error)| *index // yeah
             />
             <div> // same line comment
             // with comment on the next line
             </div>
             // comments with url: https://example.com
             <h1>"hi"</h1>
             // comments with empty lines inbetween

             // and some more
             // on the next line
                    </div>  };
            }

            // comment after view macro
        "#};

        let result = format_file_source(source, &Default::default()).unwrap();
        insta::assert_snapshot!(result, @r#"
        // comment outside view macro
        fn main() {
            view! { cx,
                // Top level comment
                <div>
                    // This is one beautiful message
                    // at the end of the line 1
                    <span>"hello"</span>
                    // at the end of the line 2
                    <div>
                        // double
                        // comments
                        <span>"hello"</span>
                    </div>
                    <For
                        // a function that returns the items we're iterating over; a signal is fine
                        each=move || { errors.clone().into_iter().enumerate() }
                        // a unique key for each item as a reference
                        // yeah
                        key=|(index, _error)| *index
                    />
                    // same line comment
                    <div>// with comment on the next line
                    </div>
                    // comments with url: https://example.com
                    <h1>"hi"</h1>
                // comments with empty lines inbetween

                // and some more
                // on the next line
                </div>
            };
        }

        // comment after view macro
        "#);
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

        let result = format_file_source(source, &Default::default()).unwrap();
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
    fn nested_with_comments() {
        let source = indoc! {r#"
            fn main() {
                view! {   cx ,  
                    // parent div
                    <div> 

                    // parent span
                    <span>{ //ok
                        let a = 12;

                        view! { cx,             
                            // wow, a span
                            <span>{a}</span>
                        }
                }</span></div>  };
            }
        "#};

        let result = format_file_source(source, &Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        fn main() {
            view! { cx,
                // parent div
                <div>

                    // parent span
                    // ok
                    <span>
                        {
                            let a = 12;

                            view! { cx,
                                // wow, a span
                                <span>{a}</span>
                            }
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

        let result = format_file_source(source, &Default::default()).unwrap();
        insta::assert_snapshot!(result, @r#"
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
        "#);
    }

    #[test]
    fn with_special_characters() {
        let source = indoc! {r#"
            fn main() {
                view! {   cx ,  <div>  <span>"hello²💣"</span></div>  }; 
            }
        "#};

        let result = format_file_source(source, &Default::default()).unwrap();
        insta::assert_snapshot!(result, @r#"
        fn main() {
            view! { cx,
                <div>
                    <span>"hello²💣"</span>
                </div>
            }; 
        }
        "#);
    }

    #[test]
    fn multiline_view_with_variable_binding() {
        let source = indoc! {r#"
        #[component]
        fn test2(cx: Scope) -> impl IntoView {
            let x = view! { cx, <div><span>Hello</span></div> };
        }
        "#};

        let result = format_file_source(source, &Default::default()).unwrap();
        insta::assert_snapshot!(result, @r###"
        #[component]
        fn test2(cx: Scope) -> impl IntoView {
            let x = view! { cx,
                <div>
                    <span>Hello</span>
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

        let result = format_file_source(source, &Default::default()).unwrap();
        insta::assert_snapshot!(result, @r#"
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
        "#);
    }

    #[test]
    fn with_unquoted_text_and_multibyte_chars() {
        let source = indoc! { r#"
            #[component]
            pub fn History() -> impl IntoView {
                // ½½½½

                view! {
                    <button>"First"</button>
                    <button>First</button>
                }
            }
        "#};

        let result = format_file_source(source, &Default::default()).unwrap();
        insta::assert_snapshot!(result, @r#"
        #[component]
        pub fn History() -> impl IntoView {
            // ½½½½

            view! {
                <button>"First"</button>
                <button>First</button>
            }
        }
        "#);
    }

    #[test]
    fn indent_with_tabs() {
        let source = indoc! {"
        fn main() {
        \u{0020}view! { cx,
              <div>
                <div>Example</div>
              </div>
            }
        }
        "};

        let result = format_file_source(
            source,
            &FormatterSettings {
                tab_spaces: 1,
                indentation_style: IndentationStyle::Tabs,
                ..Default::default()
            },
        )
        .unwrap();

        let expected = indoc! {"
        fn main() {
        \u{0020}view! { cx,
        \t\t<div>
        \t\t\t<div>Example</div>
        \t\t</div>
        \t}
        }
        "};

        assert_eq!(result, expected);
    }

    #[test]
    fn auto_detect_tabs() {
        let source = indoc! {"
        fn main() {
        \tview! { cx,
              <div>
                <div>Example</div>
              </div>
            }
        }
        "};

        let result = format_file_source(
            source,
            &FormatterSettings {
                indentation_style: IndentationStyle::Auto,
                ..Default::default()
            },
        )
        .unwrap();

        let expected = indoc! {"
        fn main() {
        \tview! { cx,
        \t\t<div>
        \t\t\t<div>Example</div>
        \t\t</div>
        \t}
        }
        "};

        assert_eq!(result, expected);
    }

    #[test]
    fn auto_detect_spaces() {
        let source = indoc! {"
        fn main() {
        \u{0020}view! { cx,
              <div>
                <div>Example</div>
              </div>
            }
        }
        "};

        let result = format_file_source(
            source,
            &FormatterSettings {
                tab_spaces: 1,
                indentation_style: IndentationStyle::Auto,
                ..Default::default()
            },
        )
        .unwrap();

        let expected = indoc! {"
        fn main() {
        \u{0020}view! { cx,
        \u{0020}\u{0020}<div>
        \u{0020}\u{0020}\u{0020}<div>Example</div>
        \u{0020}\u{0020}</div>
        \u{0020}}
        }
        "};

        assert_eq!(result, expected);
    }
}
