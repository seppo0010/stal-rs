extern crate stal;

use stal::Set::*;
use stal::Stal;

#[test]
fn noop() {
    let key = b"hello".to_vec();
    let set = Key(key.clone());
    let mut ids = vec![];
    let mut ops = vec![];
    assert_eq!(set.convert(&mut ids, &mut ops), key);
    assert_eq!(ids.len(), 0);
    assert_eq!(ops.len(), 0);
}

#[test]
fn sinter() {
    let key1 = b"hello".to_vec();
    let key2 = b"world".to_vec();
    let set = Inter(vec![Key(key1.clone()), Key(key2.clone())]);
    let mut ids = vec![];
    let mut ops = vec![];
    assert_eq!(set.convert(&mut ids, &mut ops), b"stal:0");
    assert_eq!(ids, vec!["stal:0".to_string()]);
    assert_eq!(ops, vec![vec![
            b"SINTERSTORE".to_vec(),
            b"stal:0".to_vec(),
            key1,
            key2,
            ]]);
}

#[test]
fn sunion() {
    let key1 = b"hello".to_vec();
    let key2 = b"world".to_vec();
    let set = Union(vec![Key(key1.clone()), Key(key2.clone())]);
    let mut ids = vec![];
    let mut ops = vec![];
    assert_eq!(set.convert(&mut ids, &mut ops), b"stal:0");
    assert_eq!(ids, vec!["stal:0".to_string()]);
    assert_eq!(ops, vec![vec![
            b"SUNIONSTORE".to_vec(),
            b"stal:0".to_vec(),
            key1,
            key2,
            ]]);
}

#[test]
fn sdiff() {
    let key1 = b"hello".to_vec();
    let key2 = b"world".to_vec();
    let set = Diff(vec![Key(key1.clone()), Key(key2.clone())]);
    let mut ids = vec![];
    let mut ops = vec![];
    assert_eq!(set.convert(&mut ids, &mut ops), b"stal:0");
    assert_eq!(ids, vec!["stal:0".to_string()]);
    assert_eq!(ops, vec![vec![
            b"SDIFFSTORE".to_vec(),
            b"stal:0".to_vec(),
            key1,
            key2,
            ]]);
}

#[test]
fn nesting() {
    let key1 = b"foo".to_vec();
    let key2 = b"bar".to_vec();
    let key3 = b"baz".to_vec();
    let set = Diff(vec![
            Inter(vec![
                Key(key1.clone()),
                Key(key2.clone()),
                ]),
            Key(key3.clone())
            ]);
    let mut ids = vec![];
    let mut ops = vec![];
    assert_eq!(set.convert(&mut ids, &mut ops), b"stal:0".to_vec());
    assert_eq!(ids, vec![
            "stal:0".to_string(),
            "stal:1".to_string(),
            ]);
    assert_eq!(ops, vec![
            vec![
            b"SINTERSTORE".to_vec(),
            b"stal:1".to_vec(),
            key1,
            key2,
            ],
            vec![
            b"SDIFFSTORE".to_vec(),
            b"stal:0".to_vec(),
            b"stal:1".to_vec(),
            key3,
            ],
            ]);
}

#[test]
fn explain() {
    let key1 = b"foo".to_vec();
    let key2 = b"bar".to_vec();
    let key3 = b"baz".to_vec();
    let set = Diff(vec![
            Inter(vec![
                Key(key1.clone()),
                Key(key2.clone()),
                ]),
            Key(key3.clone())
            ]);
    assert_eq!(Stal::new("SMEMBERS".to_string(), set).explain() ,vec![
            vec![
            b"SINTERSTORE".to_vec(),
            b"stal:1".to_vec(),
            key1,
            key2,
            ],
            vec![
            b"SDIFFSTORE".to_vec(),
            b"stal:0".to_vec(),
            b"stal:1".to_vec(),
            key3,
            ],
            vec![
            b"SMEMBERS".to_vec(),
            b"stal:0".to_vec(),
            ],
            ]);
}

#[test]
fn solve() {
    let key1 = b"foo".to_vec();
    let key2 = b"bar".to_vec();
    let key3 = b"baz".to_vec();
    let set = Diff(vec![
            Inter(vec![
                Key(key1.clone()),
                Key(key2.clone()),
                ]),
            Key(key3.clone())
            ]);
    assert_eq!(Stal::new("SMEMBERS".to_string(), set).solve(), (vec![
            vec![
            b"MULTI".to_vec(),
            ],
            vec![
            b"SINTERSTORE".to_vec(),
            b"stal:1".to_vec(),
            key1,
            key2,
            ],
            vec![
            b"SDIFFSTORE".to_vec(),
            b"stal:0".to_vec(),
            b"stal:1".to_vec(),
            key3,
            ],
            vec![
            b"SMEMBERS".to_vec(),
            b"stal:0".to_vec(),
            ],
            vec![
            b"DEL".to_vec(),
            b"stal:0".to_vec(),
            b"stal:1".to_vec(),
            ],
            vec![
            b"EXEC".to_vec(),
            ],
            ], 3));
}

#[test]
fn solve_noop() {
    let key1 = b"foo".to_vec();
    let set = Key(key1.clone());
    assert_eq!(Stal::new("SMEMBERS".to_string(), set).solve(), (vec![
            vec![
            b"MULTI".to_vec(),
            ],
            vec![
            b"SMEMBERS".to_vec(),
            key1,
            ],
            vec![
            b"EXEC".to_vec(),
            ],
            ], 1));
}

#[test]
fn ids() {
    let key1 = b"foo".to_vec();
    let set = Key(key1.clone());
    assert_eq!(set.ids().solve(), (vec![
            vec![
            b"MULTI".to_vec(),
            ],
            vec![
            b"SMEMBERS".to_vec(),
            key1,
            ],
            vec![
            b"EXEC".to_vec(),
            ],
            ], 1));
}

#[test]
fn nested_ids() {
    let key1 = b"foo".to_vec();
    let key2 = b"bar".to_vec();
    let key3 = b"baz".to_vec();
    let set = Diff(vec![
            Inter(vec![
                Key(key1.clone()),
                Key(key2.clone()),
                ]),
            Key(key3.clone())
            ]);
    assert_eq!(set.ids().solve(), (vec![
            vec![
            b"MULTI".to_vec(),
            ],
            vec![
            b"SINTERSTORE".to_vec(),
            b"stal:0".to_vec(),
            key1,
            key2,
            ],
            vec![
            b"SDIFF".to_vec(),
            b"stal:0".to_vec(),
            key3,
            ],
            vec![
            b"DEL".to_vec(),
            b"stal:0".to_vec(),
            ],
            vec![
            b"EXEC".to_vec(),
            ],
            ], 2));
}

#[test]
fn nested_into_ids() {
    let key1 = b"foo".to_vec();
    let key2 = b"bar".to_vec();
    let key3 = b"baz".to_vec();
    let set = Diff(vec![
            Inter(vec![
                Key(key1.clone()),
                Key(key2.clone()),
                ]),
            Key(key3.clone())
            ]);
    assert_eq!(set.into_ids().solve(), (vec![
            vec![
            b"MULTI".to_vec(),
            ],
            vec![
            b"SINTERSTORE".to_vec(),
            b"stal:0".to_vec(),
            key1,
            key2,
            ],
            vec![
            b"SDIFF".to_vec(),
            b"stal:0".to_vec(),
            key3,
            ],
            vec![
            b"DEL".to_vec(),
            b"stal:0".to_vec(),
            ],
            vec![
            b"EXEC".to_vec(),
            ],
            ], 2));
}
