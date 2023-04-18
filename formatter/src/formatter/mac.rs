use proc_macro2::{token_stream, TokenStream, TokenTree};
use syn::{spanned::Spanned, Macro};
use syn_rsx::Node;

use super::{Formatter, FormatterSettings};

impl Formatter {
    pub fn view_macro(&mut self, mac: &Macro) {
        let mut tokens = mac.tokens.clone().into_iter();
        let (Some(cx), Some(_comma)) = (tokens.next(), tokens.next()) else { return; };

        let span_start = mac.path.span().start();
        let indent = span_start.column as isize;

        let Some((tokens, global_class)) = extract_global_class(tokens) else { return; };
        let nodes = syn_rsx::parse2(tokens).unwrap_or_else(|_| {
            panic!(
                "invalid rsx tokens at line: {}:{}",
                span_start.line, span_start.column
            )
        });

        self.printer.cbox(indent);
        self.printer.word("view! { ");
        self.printer.word(cx.to_string());
        self.printer.word(",");

        if let Some(global_class) = global_class {
            self.printer.word(" class=");
            self.printer.word(global_class.to_string());
            self.printer.word(",");
        }

        self.view_macro_nodes(nodes);
        self.printer.word("}");
        self.printer.end();
    }

    fn view_macro_nodes(&mut self, nodes: Vec<Node>) {
        self.printer.cbox_indent();
        self.printer.space();

        let mut iter = nodes.iter().peekable();
        while let Some(node) = iter.next() {
            self.node(node);

            if iter.peek().is_some() {
                self.printer.hardbreak();
            }
        }

        self.printer.space();
        self.printer.end_dedent();
    }
}

fn extract_global_class(
    mut tokens: token_stream::IntoIter,
) -> Option<(TokenStream, Option<TokenTree>)> {
    let first = tokens.next();
    let second = tokens.next();
    let third = tokens.next();
    let fourth = tokens.next();
    let global_class = match (&first, &second) {
        (Some(TokenTree::Ident(first)), Some(TokenTree::Punct(eq)))
            if *first == "class" && eq.as_char() == '=' =>
        {
            match &fourth {
                Some(TokenTree::Punct(comma)) if comma.as_char() == ',' => third.clone(),
                _ => {
                    return None;
                }
            }
        }
        _ => None,
    };

    let tokens = if global_class.is_some() {
        tokens.collect::<proc_macro2::TokenStream>()
    } else {
        [first, second, third, fourth]
            .into_iter()
            .flatten()
            .chain(tokens)
            .collect()
    };

    Some((tokens, global_class))
}

pub fn format_macro(mac: &Macro, settings: FormatterSettings) -> String {
    let mut formatter = Formatter::new(settings);
    formatter.view_macro(mac);
    formatter.printer.eof()
}

#[cfg(test)]
mod tests {
    use super::format_macro;
    use quote::quote;
    use syn::Macro;

    macro_rules! view_macro {
        ($($tt:tt)*) => {{
            let mac: Macro = syn::parse2(quote! { $($tt)* }).unwrap();
            format_macro(&mac, Default::default())
        }}
    }

    #[test]
    fn one_liner() {
        let formatted = view_macro!(view! { cx, <div>"hi"</div> });
        insta::assert_snapshot!(formatted, @r###"view! { cx, <div>"hi"</div> }"###);
    }

    #[test]
    fn with_nested_nodes() {
        let formatted = view_macro!(view! { cx, <div><span>"hi"</span></div> });
        insta::assert_snapshot!(formatted, @r###"
        view! { cx,
            <div>
                <span>"hi"</span>
            </div>
        }
        "###);
    }

    #[test]
    fn with_global_class() {
        let formatted = view_macro!(view! { cx, class = STYLE, <div><span>"hi"</span></div> });
        insta::assert_snapshot!(formatted, @r###"
        view! { cx, class=STYLE,
            <div>
                <span>"hi"</span>
            </div>
        }
        "###);
    }
}
