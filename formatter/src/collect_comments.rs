use std::collections::HashMap;

use crop::{Rope, RopeSlice};

use proc_macro2::{LineColumn, Span, TokenStream, TokenTree};

use crate::line_column_to_byte;

pub(crate) fn extract_whitespace_and_comments(
    source: &Rope,
    tokens: TokenStream,
) -> HashMap<usize, Option<String>> {
    let mut whitespace_and_comments = HashMap::new();
    let mut last_span: Option<Span> = None;

    traverse_token_stream(tokens, &mut |token: &TokenTree| {
        let span = token.span();
        if let Some(last_span) = last_span {
            if last_span.end().line != span.start().line {
                let text = get_text_beween_spans(source, last_span.end(), span.start());
                for (idx, line) in text.lines().enumerate() {
                    let comment = line
                        .to_string()
                        .split_once("//")
                        .map(|(_, txt)| txt)
                        .map(str::trim)
                        .map(ToOwned::to_owned);

                    let line_index = last_span.end().line - 1 + idx;

                    if comment.is_none()
                        && (line_index == last_span.end().line - 1
                            || line_index == span.start().line - 1)
                    {
                        continue;
                    }

                    whitespace_and_comments.insert(line_index, comment);
                }
            }
        }
        last_span = Some(span);
    });

    whitespace_and_comments
}

fn get_text_beween_spans(rope: &Rope, start: LineColumn, end: LineColumn) -> RopeSlice<'_> {
    let start_byte = line_column_to_byte(rope, start);
    let end_byte = line_column_to_byte(rope, end);

    return rope.byte_slice(start_byte..end_byte);
}

fn traverse_token_stream(tokens: TokenStream, cb: &mut impl FnMut(&TokenTree)) {
    for token in tokens {
        match token {
            proc_macro2::TokenTree::Group(group) => traverse_token_stream(group.stream(), cb),
            _ => cb(&token),
        }
    }
}
