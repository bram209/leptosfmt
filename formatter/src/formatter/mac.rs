use proc_macro2::TokenStream;
use syn::{spanned::Spanned, Macro};

use super::{Formatter, FormatterSettings};

impl Formatter {
    pub fn view_macro(&mut self, mac: &Macro) {
        let mut tokens = mac.tokens.clone().into_iter();
        let (Some(cx), Some(_comma)) = (tokens.next(), tokens.next()) else { return; };
        let indent = mac.path.span().start().column as isize;

        self.printer.cbox(indent);
        // self.printer.word(" ".repeat(indent as usize));
        self.printer.word("view! { ");
        self.printer.word(cx.to_string());
        self.printer.word(",");
        self.view_macro_nodes(tokens.collect());
        self.printer.word("}");
        self.printer.end();
        // self.printer.offset(-indent);
    }

    fn view_macro_nodes(&mut self, tokens: TokenStream) {
        self.printer.cbox_indent();
        self.printer.space();

        let nodes = syn_rsx::parse2(tokens).unwrap(); // TODO error handling
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
}
