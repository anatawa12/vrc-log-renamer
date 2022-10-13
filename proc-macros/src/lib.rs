// VRC Log Renamer / proc-macros: an utility proc macro
//
// MIT License
// 
// Copyright (c) 2022 anatawa12
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate alloc;

use proc_macro2::*;
use std::collections::VecDeque;
use syn::ext::IdentExt;
use syn::{Error, Lit};

///
/// Concat identifiers or integers connected with '##'.
/// abc##def => abcdef
/// integers are not allowed at head
#[proc_macro]
pub fn concat_ident(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    process_token_stream(item.into())
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn process_token_stream(stream: TokenStream) -> Result<TokenStream, Error> {
    let mut result = TokenStream::new();
    let mut buffer = VecDeque::with_capacity(3);

    for mut t in stream {
        // fill buffer to 3
        if buffer.len() < 3 {
            buffer.push_back(t);
            continue;
        }
        // now buffer[0..=2] is safe to access
        // COMMENT OUTED PART: IDK why, but ## will be # # in proc process so removed spacing check.
        if matches!(buffer[1], TokenTree::Punct(ref p) if p.as_char() == '#'/* && p.spacing() == Spacing::Joint*/)
            && matches!(buffer[2], TokenTree::Punct(ref p) if p.as_char() == '#')
        {
            // we found ##. parse before & after
            let (mut str0, span0) = match parse_may_ident_part(&buffer[0], true) {
                Some(p) => p,
                None => return Err(Error::new(buffer[0].span(), "invalid identifier")),
            };
            let (str1, span1) = match parse_may_ident_part(&t, false) {
                Some(p) => p,
                None => return Err(Error::new(t.span(), "invalid identifier")),
            };
            // then, replace current tree & clear buffer
            str0.push_str(&str1);
            t = TokenTree::Ident(Ident::new(&str0, span0.join(span1).unwrap_or(span0)));
            buffer.clear();
        }

        // finally, put
        if let Some(popped) = buffer.pop_front() {
            result.extend(Some(process_token(popped)?));
        }
        buffer.push_back(t);
    }
    //
    while let Some(popped) = buffer.pop_front() {
        result.extend(Some(process_token(popped)?));
    }

    Ok(result)
}

fn parse_may_ident_part(tree: &TokenTree, start: bool) -> Option<(String, Span)> {
    match try_expand_variable(&tree).as_ref().unwrap_or(tree) {
        TokenTree::Ident(i) => Some((i.unraw().to_string(), i.span())),
        TokenTree::Punct(p) if p.as_char() == '_' => Some(("_".to_string(), p.span())),
        TokenTree::Literal(l) if !start => match Lit::new(l.clone()) {
            Lit::Int(l) => Some((l.base10_digits().to_string(), l.span())),
            _ => None,
        },
        TokenTree::Group(_) => None,
        _ => None,
    }
}

fn process_token(tree: TokenTree) -> Result<TokenTree, Error> {
    match tree {
        TokenTree::Group(ref g) if g.delimiter() != Delimiter::None => Ok(TokenTree::Group(
            Group::new(g.delimiter(), process_token_stream(g.stream())?),
        )),
        t => Ok(t),
    }
}

fn try_expand_variable(tree: &TokenTree) -> Option<TokenTree> {
    if let TokenTree::Group(group) = tree {
        let mut stream = group.stream().into_iter();
        let token = stream.next()?;
        if stream.next().is_none() {
            return Some(try_expand_variable(&token).unwrap_or(token));
        }
    }
    None
}
