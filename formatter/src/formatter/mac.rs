use crop::Rope;
use leptosfmt_pretty_printer::Printer;
use proc_macro2::{token_stream, Span, TokenStream, TokenTree};
use rstml::node::Node;
use syn::{spanned::Spanned, Macro};

use super::{Formatter, FormatterSettings};

pub struct ViewMacro<'a> {
    pub parent_ident: Option<usize>,
    pub cx: Option<TokenTree>,
    pub global_class: Option<TokenTree>,
    pub nodes: Vec<Node>,
    pub span: Span,
    pub mac: &'a Macro,
    pub comma: Option<TokenTree>,
}

impl<'a> ViewMacro<'a> {
    pub fn try_parse(parent_ident: Option<usize>, mac: &'a Macro) -> Option<Self> {
        let mut tokens = mac.tokens.clone().into_iter();
        let (cx, comma) = (tokens.next(), tokens.next());

        let mut no_explicit_scope = true;
        dbg!(&cx, &comma);

        // If the second token is not a comma, then leptos 0.5+ is being used, where reactive scope does not have to be manually specified.
        if let Some(TokenTree::Punct(punct)) = &comma {
            if punct.as_char() == ',' {
                no_explicit_scope = false;
            }
        };

        let (cx, comma) = if no_explicit_scope {
            tokens = [cx, comma]
                .into_iter()
                .flatten()
                .chain(tokens)
                .collect::<TokenStream>()
                .into_iter();
            (None, None)
        } else {
            (cx, comma)
        };

        let Some((tokens, global_class)) = extract_global_class(tokens) else { return None; };

        let span = mac.span();
        let nodes = rstml::parse2(tokens).ok()?;
        dbg!(&nodes);

        Some(Self {
            parent_ident,
            global_class,
            nodes,
            span,
            mac,
            cx,
            comma,
        })
    }

    pub fn inner(&self) -> &Macro {
        self.mac
    }
}

impl Formatter<'_> {
    pub fn view_macro(&mut self, view_mac: &ViewMacro) {
        let ViewMacro {
            parent_ident,
            cx,
            global_class,
            nodes,
            ..
        } = view_mac;

        let indent = parent_ident
            .map(|i| i + self.settings.tab_spaces)
            .unwrap_or(0);

        self.printer.cbox(indent as isize);

        self.flush_comments(cx.span().start().line - 1);
        self.printer.word("view! {");

        if let Some(cx) = cx {
            self.printer.word(" ");
            self.printer.word(cx.to_string());
            self.printer.word(",");
        }

        if let Some(global_class) = global_class {
            self.printer.word(" class=");
            self.printer.word(global_class.to_string());
            self.printer.word(",");
        }

        self.trim_whitespace(nodes.first().span().start().line - 1);
        self.view_macro_nodes(nodes);
        self.printer.word("}");
        self.printer.end();
    }

    fn view_macro_nodes(&mut self, nodes: &[Node]) {
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

pub fn format_macro(
    mac: &ViewMacro,
    settings: &FormatterSettings,
    source: Option<&Rope>,
) -> String {
    let mut printer = Printer::new(settings.into());
    let mut formatter = match source {
        Some(source) => {
            let whitespace = crate::collect_comments::extract_whitespace_and_comments(
                source,
                mac.mac.tokens.clone(),
            );
            Formatter::with_source(*settings, &mut printer, source, whitespace)
        }
        None => Formatter::new(*settings, &mut printer),
    };

    formatter.view_macro(mac);
    printer.eof()
}

#[cfg(test)]
mod tests {
    use super::format_macro;
    use super::ViewMacro;
    use quote::quote;
    use syn::Macro;

    macro_rules! view_macro {
        ($($tt:tt)*) => {{
            let mac: Macro = syn::parse2(quote! { $($tt)* }).unwrap();
            format_macro(&ViewMacro::try_parse(None, &mac).unwrap(), &Default::default(), None)
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

    #[test]
    fn no_reactive_scope() {
        let formatted = view_macro!(view! { <div><span>"hi"</span></div> });
        insta::assert_snapshot!(formatted, @r###"
        view! {
            <div>
                <span>"hi"</span>
            </div>
        }
        "###);
    }

    #[test]
    fn no_reactive_scope_with_global_class() {
        let formatted = view_macro!(view! { class = STYLE, <div><span>"hi"</span></div> });
        insta::assert_snapshot!(formatted, @r###"
        view! { class=STYLE,
            <div>
                <span>"hi"</span>
            </div>
        }
        "###);
    }
}
