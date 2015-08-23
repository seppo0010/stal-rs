#![crate_name = "stal"]
#![crate_type = "lib"]

pub struct Hello;

pub enum Stal {
    Key(Vec<u8>),
    Union(Vec<Stal>),
    Inter(Vec<Stal>),
    Diff(Vec<Stal>),
}
use Stal::*;

impl Stal {
    fn command(&self, root: bool) -> &'static str {
        match *self {
            Key(_) => unreachable!(),
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
