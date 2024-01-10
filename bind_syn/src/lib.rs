//! # Why This Project
//!
//! Provides `enum Bind`, the syntax definition of let-binding shorthand utilized
//! in `crate bind`.
//!
//! This is not a proc-macro library, but a library providing syntax parsing for
//! those proc macro libraries which provide similar functionality with
//! `crate bind`.

use quote::{ToTokens, quote};

use syn::{
    Expr,
    ExprAssign,
    ExprPath,
    Ident,
    Token,
    parse::{self, Parse, ParseStream},
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

/// The definition of let-binding shorthands.
pub enum Bind {
    /// generates `let id = id.clone();`
       Id(     Ident              ),
    /// generates `let mut id = id.clone();`
    MutId(     Ident              ),
    /// generates `let id = id0.clone();`
       IdId(   Ident, Ident       ),
    /// generates `let mut id = id0.clone();`
    MutIdId(   Ident, Ident       ),
    /// generates `let id = expr;`
       IdExpr( Ident,        Expr ),
    /// generates `let mut id = expr;`
    MutIdExpr( Ident,        Expr ),
    /// generates `let id_extracted_from_expr = expr;`
         Expr( Ident,        Expr ),
    /// generates `let mut id_extracted_from_expr = expr;`
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
