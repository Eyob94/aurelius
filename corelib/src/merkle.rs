#![allow(unused)]
use borsh::{BorshDeserialize, BorshSerialize};

type Hash = [u8; 32];

#[derive(Debug,PartialEq, Eq, Default, Clone, BorshDeserialize, BorshSerialize)]
pub struct Node {
    pub hash: Hash,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

impl Node {
    pub fn new() -> Self {
        Self {
            hash: [0u8; 32],
            left: None,
            right: None,
        }
    }

    pub fn with_hash(hash: Hash) -> Self {
        Self {
            hash,
            left: None,
            right: None,
        }
    }

    pub fn from_children(left: Node, right: Node) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&left.hash);
        hasher.update(&right.hash);
        let hash = *hasher.finalize().as_bytes();

        Self {
            hash,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Tree {
    root: Option<Node>,
}

impl Tree {
    pub fn new() -> Self {
        Self { root: None }
    }
    pub fn with_hashes(hashes: &[Hash]) -> Self {
        let mut tree = Tree::new();
        tree.build_tree(hashes);

        tree
    }

    pub fn build_tree<T>(&mut self, hashes: T)
    where
        T: AsRef<[Hash]>,
    {
        let mut nodes: Vec<Node> = hashes
            .as_ref()
            .iter()
            .map(|h| Node::with_hash(*h))
            .collect();

        self.root = Tree::build(nodes).map(|n| *n);
    }

    pub fn build<T: AsRef<[Node]>>(nodes: T) -> Option<Box<Node>> {
        let nodes = nodes.as_ref();
        if nodes.is_empty() {
            return None;
        }
        if nodes.len() == 1 {
            return Some(Box::new(nodes[0].clone()));
        }

        if nodes.len() == 2 {
            return Some(Box::new(Node::from_children(
                nodes[0].clone(),
                nodes[1].clone(),
            )));
        }

        let (left, right) = nodes.split_at(nodes.len() / 2);

        Some(Box::new(Node::from_children(
            *Tree::build(left).unwrap(),
            *Tree::build(right).unwrap(),
        )))
    }

    pub fn root_hash(&self) -> Option<Hash> {
        self.root.as_ref().map(|r| r.hash)
    }

    // [`leaf number`] is the index of the leaf nodes not the entire tree.
    //
    // So for example if there are 5 transactions and we want to get a proof for the
    // 3rd transaction the leaf_number will be 3 despite the node holding that
    // leaf may not have index 3
    pub fn generate_proof(&self, leaf_number: u32) -> Option<Vec<Hash>> {
        todo!()
    }

    pub fn verify_proof(leaf_hash: Hash, proof: &[Hash], root_hash: Hash) -> bool {
        todo!()
    }
}

pub struct Proof {
    hash: Hash,
    proofs: Vec<IndexedHash>,
}

pub struct IndexedHash {
    index: u32,
    hash: Hash,
}

#[cfg(test)]
mod test {
    use super::Tree;

    #[test]
    fn creates_and_proofs_tree() {
        let hashes: Vec<[u8; 32]> = vec![[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];

        let mut tree = Tree::default();

        tree.build_tree(hashes.clone());
        let root_hash = tree.root_hash();

        // TODO: test proofs
        // for (index, hash) in hashes.iter().enumerate() {
        //     let proof = tree.generate_proof(index as u32);
        //     assert!(proof.is_some(), "Proof for leaf {} should exist", index);
        //
        //     let proof = proof.unwrap();
        //     assert!(
        //         Tree::verify_proof(*hash, &proof, root_hash.unwrap()),
        //         "Proof verification for leaf {} should pass",
        //         index
        //     );
        // }
    }
}
