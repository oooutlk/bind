/// # Why This Project
///
/// Sometimes we are forced to write some boring code like:
///
/// ```rust
/// let foo2 = foo.clone();
/// let bar2 = *bar;
/// let baz2 = baz.to_owned();
/// let f = move |args| {
///     // access to foo2, bar2 and baz2
/// };
/// ```
///
/// It's quite annoying, messing up the source code and the readers can't focus
/// on business logic. Some crates have been published to dealing with this,
/// and the bind crate is yet another one, inspired by `crate enclose`, which
/// provides a convenient declarative macro. Since crate bind is a `proc_macro`,
/// it can do more than `macro_rules`.
///
/// # Example
/// ```rust
///
/// let f = bind!( ( foo,*bar,baz.to_owned() )
///     move |args| {
///         // access to foo, bar and baz
///     }
/// );
/// ```

use proc_macro::TokenStream;

use quote::{ToTokens, quote};

use syn::{
    Expr,
    ExprAssign,
    ExprClosure,
    ExprPath,
    Ident,
    Token,
    parenthesized,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token,
    visit::Visit,
};

fn extract_the_only_id_in( expr: &Expr ) -> Option<Ident> {
    struct Extractor {
        id  : Option<Ident>,
        cnt : usize,
    }

    impl<'a> Visit<'a> for Extractor {
        fn visit_ident( &mut self, id: &Ident ) {
            if self.cnt == 0 && self.id.is_none() {
                self.id = Some( id.clone() );
            }
            self.cnt += 1;
        }
    }

    let mut extractor = Extractor{ id: None, cnt: 0 };
    extractor.visit_expr( &expr );
    extractor.id
}

enum ExprOrIdent {
    Expr(  Expr  ),
    Ident( Ident ),
}

fn get_expr_or_id( expr: Expr ) -> ExprOrIdent {
    if let Expr::Path( ExprPath{ attrs, qself, path }) = &expr {
        if attrs.is_empty() && qself.is_none() {
            if path.leading_colon.is_none() && path.segments.len() == 1 {
                let seg = path.segments.first().unwrap();
                if seg.arguments.is_none() {
                    return ExprOrIdent::Ident( seg.ident.clone() );
                }
            }
        }
    }
    ExprOrIdent::Expr( expr )
}

enum Bind {
       Id(     Ident              ),
    MutId(     Ident              ),
       IdId(   Ident, Ident       ),
    MutIdId(   Ident, Ident       ),
       IdExpr( Ident,        Expr ),
    MutIdExpr( Ident,        Expr ),
         Expr( Ident,        Expr ),
      MutExpr( Ident,        Expr ),
}

impl Parse for Bind {
    fn parse( input: ParseStream ) -> parse::Result<Self> {
        let immutable = if input.peek( Token![mut] ) {
            input.parse::<Token![mut]>()?;
            false
        } else {
            true
        };

        let expr = input.parse::<Expr>()?;

        if let Expr::Assign( expr_assign ) = &expr {
            let ExprAssign{ attrs:_, left, eq_token, right } = expr_assign.clone();
            let _ = eq_token;
            if let ExprOrIdent::Ident( id ) = get_expr_or_id( *left ) {
                match get_expr_or_id( *right ) {
                    ExprOrIdent::Expr( expr ) =>
                        return Ok( if immutable {
                            Bind::IdExpr(    id, expr )
                        } else {
                            Bind::MutIdExpr( id, expr )
                        }),
                    ExprOrIdent::Ident( id0 ) =>
                        return Ok( if immutable {
                            Bind::IdId(      id, id0 )
                        } else {
                            Bind::MutIdId(   id, id0 )
                        }),
                }
            }
        } else {
            match get_expr_or_id( expr ) {
                ExprOrIdent::Expr( expr ) =>
                    match extract_the_only_id_in( &expr ) {
                        Some( id ) =>
                            return Ok( if immutable {
                                Bind::Expr(    id, expr )
                            } else {
                                Bind::MutExpr( id, expr )
                            }),
                        None => (),
                    }
                ExprOrIdent::Ident( id ) =>
                    return Ok( if immutable {
                        Bind::Id(    id )
                    } else {
                        Bind::MutId( id )
                    }),
            }
        }

        panic!( "Invalid input for `bind!()`: {input:?}" );
    }
}

impl ToTokens for Bind {
    fn to_tokens( &self, tokens: &mut proc_macro2::TokenStream ) {
        tokens.extend( match self {
            Bind::Id(         id           ) => quote!{ let     #id = #id  .clone(); },
            Bind::MutId(      id           ) => quote!{ let mut #id = #id  .clone(); },
            Bind::IdId(       id, id0      ) => quote!{ let     #id = #id0 .clone(); },
            Bind::MutIdId(    id, id0      ) => quote!{ let mut #id = #id0 .clone(); },
            Bind::IdExpr(     id,     expr ) => quote!{ let     #id = #expr        ; },
            Bind::MutIdExpr(  id,     expr ) => quote!{ let mut #id = #expr        ; },
            Bind::Expr(       id,     expr ) => quote!{ let     #id = #expr        ; },
            Bind::MutExpr(    id,     expr ) => quote!{ let mut #id = #expr        ; },
        });
    }
}

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
