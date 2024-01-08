A proc macro for binding values into an expression (usually a closure).

# Why This Project
                                                                             
Sometimes we are forced to write some boring code like:
                                                                             
```rust
let foo2 = foo.clone();
let bar2 = *bar;
let baz2 = baz.to_owned();
let f = move |args| {
    // access to foo2, bar2 and baz2
};
```
                                                                             
It's quite annoying, messing up the source code and the readers can't focus
on business logic. Some crates have been published to dealing with this,
and the bind crate is yet another one, inspired by `crate enclose`, which
provides a convenient declarative macro. Since crate bind is a `proc_macro`,
it can do more than `macro_rules`.
                                                                             
# Example
                                                                             
```rust
let f = bind!( ( foo,*bar,baz.to_owned() )
    move |args| {
        // access to foo, bar and baz
    }
);
```

See [`bind/README.md`](bind/README.md) for more.

# License

Under Apache License 2.0 or MIT License, at your will.
