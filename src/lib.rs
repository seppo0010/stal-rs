//! stal-rs
//! ====
//!
//! Set algebra solver for Redis in Rust, based on
//! [Stal](https://github.com/soveran/stal).
//!
//! Description
//! -----------
//!
//! `stal-rs` provide set operations and resolves them in [Redis][redis].
//!
//! Usage
//! -----
//!
//! `stal-rs` has no dependencies. It produces a vector of Redis operations that
//! have to be run by the user.
//!
//! ```rust
//! extern crate stal;
//!
//! let foobar = stal::Set::Inter(vec![stal::Set::Key(b"foo".to_vec()), stal::Set::Key(b"bar".to_vec())]);
//! let foobar_nobaz = stal::Set::Diff(vec![foobar, stal::Set::Key(b"baz".to_vec())]);
//! let foobar_nobaz_andqux = stal::Set::Union(vec![stal::Set::Key(b"qux".to_vec()), foobar_nobaz]);
//!
//! assert_eq!(
//!    stal::Stal::new("SMEMBERS".to_string(), foobar_nobaz_andqux).solve(),
//!    (
//!     vec![
//!         vec![b"MULTI".to_vec()],
//!         vec![b"SINTERSTORE".to_vec(), b"stal:2".to_vec(), b"foo".to_vec(), b"bar".to_vec()],
//!         vec![b"SDIFFSTORE".to_vec(), b"stal:1".to_vec(), b"stal:2".to_vec(), b"baz".to_vec()],
//!         vec![b"SUNIONSTORE".to_vec(), b"stal:0".to_vec(), b"qux".to_vec(), b"stal:1".to_vec()],
//!         vec![b"SMEMBERS".to_vec(), b"stal:0".to_vec()],
//!         vec![b"DEL".to_vec(), b"stal:0".to_vec(), b"stal:1".to_vec(), b"stal:2".to_vec()],
//!         vec![b"EXEC".to_vec()],
//!     ],
//!     4
//!    ));
//! ```
//!
//! `stal-rs` translates the internal calls to  `SUNION`, `SDIFF` and
//! `SINTER` into `SDIFFSTORE`, `SINTERSTORE` and `SUNIONSTORE` to
//! perform the underlying operations, and it takes care of generating
//! and deleting any temporary keys.
//!
//! The outmost command can be any set operation, for example:
//!
//! ```rust
//! extern crate stal;
//! let myset = stal::Set::Key(b"my set".to_vec());
//! stal::Stal::new("SCARD".to_string(), myset).solve();
//! ```
//!
//! If you want to preview the commands `Stal` will send to generate
//! the results, you can use `Stal.explain`:
//!
//! ```rust
//! extern crate stal;
//!
//! assert_eq!(
//! stal::Stal::new("SMEMBERS".to_string(),
//!         stal::Set::Inter(vec![
//!             stal::Set::Union(vec![
//!                 stal::Set::Key(b"foo".to_vec()),
//!                 stal::Set::Key(b"bar".to_vec()),
//!                 ]),
//!             stal::Set::Key(b"baz".to_vec()),
//!             ])
//!         ).explain(),
//! vec![
//!     vec![b"SUNIONSTORE".to_vec(), b"stal:1".to_vec(), b"foo".to_vec(), b"bar".to_vec()],
//!     vec![b"SINTERSTORE".to_vec(), b"stal:0".to_vec(), b"stal:1".to_vec(), b"baz".to_vec()],
//!     vec![b"SMEMBERS".to_vec(), b"stal:0".to_vec()],
//! ]
//! )
//! ```
//!
//! All commands are wrapped in a `MULTI/EXEC` transaction.
//!
//! [redis]: http://redis.io

#![crate_name = "stal"]
#![crate_type = "lib"]

/// A set of values. It can be generated from a Redis key or from a set
/// operation based on other sets.
#[derive(Debug, Clone)]
pub enum Set {
    /// A key
    Key(Vec<u8>),
    /// All the elements in any of the provided sets
    Union(Vec<Set>),
    /// All the elements in all the sets
    Inter(Vec<Set>),
    /// All the elements in the first set that are not in the other sets
    Diff(Vec<Set>),
}
use Set::*;

impl Set {
    /// Gets the commands to get a list of ids for this set
    pub fn into_ids(self) -> Stal {
        let (op, sets) = match self {
            Key(_) => return Stal::from_template(vec![b"SMEMBERS".to_vec(), vec![]], vec![(self, 1)]),
            Union(sets) => ("SUNION", sets),
            Inter(sets) => ("SINTER", sets),
            Diff(sets) => ("SDIFF", sets),
        };
        let mut command = vec![op.as_bytes().to_vec()];
        command.extend(sets.iter().map(|_| vec![]).collect::<Vec<_>>());
        let mut setv = vec![];
        let mut i = 1;
        for set in sets.into_iter() {
            setv.push((set, i));
            i += 1;
        }
        Stal::from_template(command, setv)
    }

    /// Gets the commands to get a list of ids for this set
    pub fn ids(&self) -> Stal {
        let (op, sets) = match *self {
            Key(_) => return Stal::from_template(vec![b"SMEMBERS".to_vec(), vec![]], vec![(self.clone(), 1)]),
            Union(ref sets) => ("SUNION", sets),
            Inter(ref sets) => ("SINTER", sets),
            Diff(ref sets) => ("SDIFF", sets),
        };
        let mut command = vec![op.as_bytes().to_vec()];
        command.extend(sets.iter().map(|_| vec![]).collect::<Vec<_>>());
        let mut setv = vec![];
        for i in 0..sets.len() {
            setv.push((sets[i].clone(), i + 1));
        }
        Stal::from_template(command, setv)
    }

    /// Maps the operation to its Redis command name.
    fn command(&self) -> &'static str {
        match *self {
            Key(_) => unreachable!(),
            Union(_) => "SUNIONSTORE",
            Inter(_) => "SINTERSTORE",
            Diff(_) => "SDIFFSTORE",
        }
    }

    /// Appends the operation to `ops` and any temporary id created to `ids`.
    /// Returns the key representing the set.
    pub fn convert(&self, ids: &mut Vec<String>, ops: &mut Vec<Vec<Vec<u8>>>) -> Vec<u8> {
        let sets = match *self {
            Key(ref k) => return k.clone(),
            Union(ref sets) => sets,
            Inter(ref sets) => sets,
            Diff(ref sets) => sets,
        };

        let mut op = Vec::with_capacity(2 + sets.len());
        let id = format!("stal:{}", ids.len());
        let r = id.as_bytes().to_vec();
        ids.push(id);
        op.push(self.command().as_bytes().to_vec());
        op.push(r.clone());
        op.extend(sets.into_iter().map(|s| s.convert(ids, ops)));
        ops.push(op);
        r
    }
}

/// An operation to be executed on a set
#[derive(Debug)]
pub struct Stal {
    /// A Redis command
    command: Vec<Vec<u8>>,
    /// Set in which execute the operation
    sets: Vec<(Set, usize)>,
}

impl Stal {
    pub fn new(operation: String, set: Set) -> Self {
        Stal {
            command: vec![operation.as_bytes().to_vec(), vec![]],
            sets: vec![(set, 1)],
        }
    }

    /// Takes an arbitrary command that uses one or more sets. The `command`
    /// must have placeholders where the set keys should go. Each element
    /// in `sets` specifies the position in the `command`.
    pub fn from_template(command: Vec<Vec<u8>>, sets: Vec<(Set, usize)>) -> Self {
        Stal {
            command: command,
            sets: sets,
        }
    }

    fn add_ops(&self, ids: &mut Vec<String>, ops: &mut Vec<Vec<Vec<u8>>>) {
        let mut command = self.command.clone();
        for args in self.sets.iter() {
            command.push(args.0.convert(ids, ops));
            command.swap_remove(args.1);
        }
        ops.push(command);
    }

    /// Returns a list of operations to run. For debug only.
    pub fn explain(&self) -> Vec<Vec<Vec<u8>>> {
        let mut ids = vec![];
        let mut ops = vec![];
        self.add_ops(&mut ids, &mut ops);
        ops
    }

    /// Returns a lit of operations, wrapped in a multi/exec.
    /// The last operation is always exec, and the returned `usize` indicates
    /// the return value of the `operation`.
    pub fn solve(&self) -> (Vec<Vec<Vec<u8>>>, usize) {
        let mut ids = vec![];
        let mut ops = vec![vec![b"MULTI".to_vec()]];
        self.add_ops(&mut ids, &mut ops);
        let pos = ops.len() - 1;
        if ids.len() > 0 {
            let mut del = vec![b"DEL".to_vec()];
            del.extend(ids.into_iter().map(|x| x.as_bytes().to_vec()));
            ops.push(del);
        }
        ops.push(vec![b"EXEC".to_vec()]);
        (ops, pos)
    }
}
