This crate provides a proc macro to generate "let bindings" automatically,
usually cloning values into an expression(usually a closure). Inspired by
`crate enclose`. 

# Syntax

`bind!( ( comma_separated_list_of_var_bindings ) the_expr_that_uses_the_vars )`

`comma_separated_list_of_var_bindings` is in the form of
`var_binding, another var_binding, ...`.

`var_binding` is in the form of:

1. `id`, generating `let id = id.clone();`

2. `mut id`, generating `let mut id = id.clone();`

3. `new_id = id`, generating `let new_id = id.clone();`

4. `mut new_id = id`, generating `let mut new_id = id.clone();`

5. `id = expr`, generating `let id = expr;`

6. `mut id = expr`, generating `let mut id = expr;`

7. `expr`, generating `let the_only_id_in_the_expr = expr;`,
    e.g. `bind!( (s.to_owned()) .. )` generates `let s = s.to_owned()`.

8. `mut expr`, generating `let mut the_only_id_in_the_expr = expr;`
    e.g. `bind!( (mut s.to_owned()) .. )` generates `let mut s = s.to_owned()`.
