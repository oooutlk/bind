//! # Why This Project
//!
//! Sometimes we are forced to write some boring code like:
//!
//! ```rust
//! let foo2 = foo.clone();
//! let bar2 = *bar;
//! let baz2 = baz.to_owned();
//! let f = move |args| {
//!     // access to foo2, bar2 and baz2
//! };
//! ```
//!
//! It's quite annoying, messing up the source code and the readers can't focus
//! on business logic. Some crates have been published to dealing with this,
//! and the bind crate is yet another one, inspired by `crate enclose`, which
//! provides a convenient declarative macro. Since crate bind is a `proc_macro`,
//! it can do more than `macro_rules`.
//!
//! # Example
//! ```rust
//!
//! let f = bind!( ( foo,*bar,baz.to_owned() )
//!     move |args| {
//!         // access to foo, bar and baz
//!     }
//! );
//! ```

use bind_syn::Bind;

use proc_macro::TokenStream;

use quote::quote;

use syn::{
    Expr,
    ExprClosure,
    Token,
    parenthesized,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token,
};

struct BindInput {
    paren : token::Paren,
    binds : Punctuated<Bind,Token![,]>,
    expr  : Expr,
}

impl Parse for BindInput {
    fn parse( input: ParseStream ) -> parse::Result<Self> {
        let content;
        let paren = parenthesized!( content in input );
        let binds = Punctuated::parse_terminated( &content )?;
        let expr = input.parse::<Expr>()?;
        Ok( BindInput{ paren, binds, expr })
    }
}

/// A proc macro to generate "let bindings" automatically, usually cloning values into an expression(usually a closure).
/// Inspired by `crate enclose`.
///
/// # Syntax
///
/// bind!( ( a_comma_separated_list_of_var_bindings ) the_expr_that_uses_the_vars )
///
/// `a_comma_separated_list_of_var_bindings` is in the form of `var_binding, another var_binding, ...`.
///
/// `var_binding` is in the form of:
///
/// 1. `id`, generating `let id = id.clone();`
///
/// 2. `mut id`, generating `let mut id = id.clone();`
///
/// 3. `new_id = id`, generating `let new_id = id.clone();`
///
/// 4. `mut new_id = id`, generating `let mut new_id = id.clone();`
///
/// 5. `id = expr`, generating `let id = expr;`
///
/// 6. `mut id = expr`, generating `let mut id = expr;`
///
/// 7. `expr`, generating `let the_only_id_in_the_expr = expr;`,
///     e.g. `bind!( (s.to_owned()) .. )` generates `let s = s.to_owned()`.
///
/// 8. `mut expr`, generating `let mut the_only_id_in_the_expr = expr;`
///     e.g. `bind!( (mut s.to_owned()) .. )` generates `let mut s = s.to_owned()`.
#[proc_macro]
pub fn bind( input: TokenStream ) -> TokenStream {
    let BindInput{ paren, binds, expr } = parse_macro_input!( input as BindInput );
    let _ = paren;
    let binds = binds.iter();
    let extrusive = if let Expr::Closure( expr_closure ) = &expr {
        expr_closure.capture.is_some()
    } else {
        true
    };

    let expanded = if extrusive {
        quote!{{
            #(#binds)*
            #expr
        }}
    } else {
        if let Expr::Closure( ExprClosure{ attrs, lifetimes, constness, movability, asyncness,
            capture, or1_token, inputs, or2_token, output, body })
            = expr {
            quote!{{
                #(#attrs)* #lifetimes #constness #movability #asyncness
                #capture #or1_token #inputs #or2_token #output {
                    #(#binds)*
                    #body
                }
            }}
        } else {
            unreachable!();
        }
    };

    expanded.into()
}
