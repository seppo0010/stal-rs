#![crate_name = "stal"]
#![crate_type = "lib"]

pub enum Set {
    Key(Vec<u8>),
    Union(Vec<Set>),
    Inter(Vec<Set>),
    Diff(Vec<Set>),
}
use Set::*;

impl Set {
    fn command(&self) -> &'static str {
        match *self {
            Key(_) => unreachable!(),
            Union(_) => "SUNIONSTORE",
            Inter(_) => "SINTERSTORE",
            Diff(_) => "SDIFFSTORE",
        }
    }

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

pub struct Stal {
    operation: String,
    set: Set,
}

impl Stal {
    pub fn new(operation: String, set: Set) -> Self {
        Stal {
            operation: operation,
            set: set,
        }
    }

    pub fn explain(&self) -> Vec<Vec<Vec<u8>>> {
        let mut ids = vec![];
        let mut ops = vec![];
        let key = self.set.convert(&mut ids, &mut ops);
        ops.push(vec![self.operation.as_bytes().to_vec(), key]);
        ops
    }

    pub fn solve(&self) -> (Vec<Vec<Vec<u8>>>, usize) {
        let mut ids = vec![];
        let mut ops = vec![vec![b"MULTI".to_vec()]];
        let key = self.set.convert(&mut ids, &mut ops);
        ops.push(vec![self.operation.as_bytes().to_vec(), key]);
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
