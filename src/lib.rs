pub enum Set {
    Key(Vec<u8>),
    Union(Vec<Set>),
    Inter(Vec<Set>),
    Diff(Vec<Set>),
}
use Set::*;

impl Set {
    fn command(&self, root: bool) -> &'static str {
        match *self {
            Key(_) => panic!("Getting command for a Key Set"),
            Union(_) => if root { "SUNION" } else { "SUNIONSTORE" },
            Inter(_) => if root { "SINTER" } else { "SINTERSTORE" },
            Diff(_) => if root { "SDIFF" } else { "SDIFFSTORE" },
        }
    }

    fn conv(&self, ids: &mut Vec<String>, ops: &mut Vec<Vec<Vec<u8>>>, root: bool) -> Vec<u8> {
        let sets = match *self {
            Key(ref k) => return k.clone(),
            Union(ref sets) => sets,
            Inter(ref sets) => sets,
            Diff(ref sets) => sets,
        };

        let mut op = Vec::with_capacity(2 + sets.len());
        let r = if !root {
            let id = format!("stal:{}", ids.len());
            let r = id.as_bytes().to_vec();
            ids.push(id);
            op.push(self.command(false).as_bytes().to_vec());
            op.push(r.clone());
            r
        } else {
            op.push(self.command(true).as_bytes().to_vec());
            vec![]
        };
        op.extend(sets.into_iter().map(|s| s.conv(ids, ops, false)));
        ops.push(op);
        r
    }

    pub fn convert(&self, ids: &mut Vec<String>, ops: &mut Vec<Vec<Vec<u8>>>) -> Vec<u8> {
        self.conv(ids, ops, true)
    }

    pub fn explain(&self) -> Vec<Vec<Vec<u8>>> {
        let mut ids = vec![];
        let mut ops = vec![];
        self.convert(&mut ids, &mut ops);
        ops
    }

    pub fn solve(&self) -> (Vec<Vec<Vec<u8>>>, usize) {
        let mut ids = vec![];
        let mut ops = vec![vec![b"MULTI".to_vec()]];
        let key = self.convert(&mut ids, &mut ops);
        if ops.len() == 1 {
            ops.push(vec![b"SMEMBERS".to_vec(), key]);
        }
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
    assert_eq!(set.convert(&mut ids, &mut ops), vec![]);
    assert_eq!(ids.len(), 0);
    assert_eq!(ops, vec![vec![
            b"SINTER".to_vec(),
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
    assert_eq!(set.convert(&mut ids, &mut ops), vec![]);
    assert_eq!(ids.len(), 0);
    assert_eq!(ops, vec![vec![
            b"SUNION".to_vec(),
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
    assert_eq!(set.convert(&mut ids, &mut ops), vec![]);
    assert_eq!(ids.len(), 0);
    assert_eq!(ops, vec![vec![
            b"SDIFF".to_vec(),
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
    assert_eq!(set.convert(&mut ids, &mut ops).len(), 0);
    assert_eq!(ids, vec![
            "stal:0".to_string(),
            ]);
    assert_eq!(ops, vec![
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
    assert_eq!(set.explain() ,vec![
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
    assert_eq!(set.solve(), (vec![
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
fn solve_noop() {
    let key1 = b"foo".to_vec();
    let set = Key(key1.clone());
    assert_eq!(set.solve(), (vec![
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
