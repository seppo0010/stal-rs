stal-rs
====

Set algebra solver for Redis in Rust, based on
[Stal](https://github.com/soveran/stal).

Description
-----------

`stal-rs` provide set operations and resolves them in [Redis][redis].

Usage
-----

`stal-rs` has no dependencies. It produces a vector of Redis operations that
have to be run by the user.

```rust
extern crate stal;

let foobar = stal::Set::Inter(vec![stal::Set::Key(b"foo".to_vec()), stal::Set::Key(b"bar".to_vec())]);
let foobar_nobaz = stal::Set::Diff(vec![foobar, stal::Set::Key(b"baz".to_vec())]);
let foobar_nobaz_andqux = stal::Set::Union(vec![stal::Set::Key(b"qux".to_vec()), foobar_nobaz]);

assert_eq!(
    stal::Stal::new("SMEMBERS".to_string(), foobar_nobaz_andqux).solve(),
    (
     vec![
         vec![b"MULTI".to_vec()],
         vec![b"SINTERSTORE".to_vec(), b"stal:2".to_vec(), b"foo".to_vec(), b"bar".to_vec()],
         vec![b"SDIFFSTORE".to_vec(), b"stal:1".to_vec(), b"stal:2".to_vec(), b"baz".to_vec()],
         vec![b"SUNIONSTORE".to_vec(), b"stal:0".to_vec(), b"qux".to_vec(), b"stal:1".to_vec()],
         vec![b"SMEMBERS".to_vec(), b"stal:0".to_vec()],
         vec![b"DEL".to_vec(), b"stal:0".to_vec(), b"stal:1".to_vec(), b"stal:2".to_vec()],
         vec![b"EXEC".to_vec()],
     ],
     4
    ));
```

`stal-rs` translates the internal calls to  `SUNION`, `SDIFF` and
`SINTER` into `SDIFFSTORE`, `SINTERSTORE` and `SUNIONSTORE` to
perform the underlying operations, and it takes care of generating
and deleting any temporary keys.

The outmost command can be any set operation, for example:

```rust
extern crate stal;
let myset = stal::Set::Key(b"my set".to_vec());
stal::Stal::new("SCARD".to_string(), myset).solve();
```

If you want to preview the commands `Stal` will send to generate
the results, you can use `Stal.explain`:

```rust
extern crate stal;

assert_eq!(
stal::Stal::new("SMEMBERS".to_string(),
        stal::Set::Inter(vec![
            stal::Set::Union(vec![
                stal::Set::Key(b"foo".to_vec()),
                stal::Set::Key(b"bar".to_vec()),
                ]),
            stal::Set::Key(b"baz".to_vec()),
            ])
        ).explain(),
vec![
    vec![b"SUNIONSTORE".to_vec(), b"stal:1".to_vec(), b"foo".to_vec(), b"bar".to_vec()],
    vec![b"SINTERSTORE".to_vec(), b"stal:0".to_vec(), b"stal:1".to_vec(), b"baz".to_vec()],
    vec![b"SMEMBERS".to_vec(), b"stal:0".to_vec()],
]
)
```

All commands are wrapped in a `MULTI/EXEC` transaction.

Installation
------------

```toml
[dependencies]
stal = "0.1.0"
```

[redis]: http://redis.io
