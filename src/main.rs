use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashMap},
    ptr,
};

use bitvec::vec::BitVec;

// iosevka
// fira code ? (firefox)

fn main() {
    // let input = r"hello world!".as_bytes();
    // let tree = Tree::build_tree(input).unwrap();
    // println!("{:#?}", tree);

    let input_str: &str = "hello 😊 xyz 🙋🏿‍♀️ world!";
    // println!("{input_str}");
    // dbg!(input_str);
    let input: &[u8] = input_str.as_bytes();
    // dbg!(input);  // ??
    // // println!("{input:?}");
    // return;

    let (tree, compressed) = huffman_compress(input).unwrap();
    dbg!(&compressed);
    dbg!(input.len() * 8);
    let decompressed = huffman_decompress(&tree, &compressed).unwrap();
    let out = String::from_utf8_lossy(&decompressed);
    println!("{}", out);
}

/*
input: ASCII text, fixed input 'const' Vec<u8>

#1
freqs: HashMap<u8, usize>

#2
(optimal) tree / code-mapping, based on the freqs
* high-freq things have shorter codes

#3
2nd pass thru input, apply mapping

output:  sequence of bits
*/

// todo++: general input elements (not just 'ascii' u8)

pub fn huffman_compress(input: &[u8]) -> Option<(Tree, BitVec)> {
    let tree = Tree::build_tree(input)?;
    let output = tree.compress(input)?;
    Some((tree, output))
}

pub fn huffman_decompress(tree: &Tree, input: &BitVec) -> Option<Vec<u8>> {
    tree.decompress(input)
}

// todo: serializable Tree
// * how to represent?
//   * serde?
//   * as bits somehow?
//   * somethng standard? (name of standard = ?)

#[derive(Debug)]
pub struct Tree {
    root: Node,
    symbol_encodings: HashMap<u8, BitVec>,
}

#[derive(Debug)]
enum Node {
    Branch {
        children: [Box<Node>; 2],
        // left: Box<Node>,
        // right: Box<Node>,
    },
    Leaf {
        symbol: u8,
    },
}

struct HeapElem {
    freq: usize,
    node: Box<Node>,
}

impl Ord for HeapElem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.freq.cmp(&other.freq)
    }
}

impl PartialOrd for HeapElem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HeapElem {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Eq for HeapElem {}

impl Tree {
    fn build_tree(input: &[u8]) -> Option<Self> {
        if input.is_empty() {
            return None;
        }

        let mut freqs = HashMap::<u8, usize>::with_capacity(256);
        for &b in input {
            *freqs.entry(b).or_default() += 1;
        }

        let mut nodes: BinaryHeap<_> = freqs
            .into_iter()
            .map(|(symbol, freq)| {
                Reverse(HeapElem {
                    freq,
                    node: Box::new(Node::Leaf { symbol }),
                })
            })
            .collect();

        while nodes.len() >= 2 {
            // combine the two smallest
            let Reverse(elem1) = nodes.pop().unwrap();
            let Reverse(elem2) = nodes.pop().unwrap();

            let freq = elem1.freq + elem2.freq;
            let node = Box::new(Node::Branch {
                children: [elem1.node, elem2.node],
            });
            nodes.push(Reverse(HeapElem { freq, node }));
        }

        let Reverse(HeapElem { node: root, .. }) = nodes.pop().unwrap();
        let mut symbol_encodings = HashMap::new();
        Self::construct_symbol_encodings(&root, &mut BitVec::new(), &mut symbol_encodings);
        Some(Self {
            root: *root,
            symbol_encodings,
        })
    }

    fn construct_symbol_encodings(node: &Node, path: &mut BitVec, map: &mut HashMap<u8, BitVec>) {
        match node {
            Node::Branch { children: [x, y] } => {
                path.push(false);
                Self::construct_symbol_encodings(x, path, map);
                path.pop();
                path.push(true);
                Self::construct_symbol_encodings(y, path, map);
                path.pop();
            }
            Node::Leaf { symbol } => {
                map.insert(*symbol, path.clone());
            }
        }
    }

    fn compress(&self, input: &[u8]) -> Option<BitVec> {
        let mut out = BitVec::new();
        for byte in input {
            let code = self.symbol_encodings.get(byte)?;
            out.extend_from_bitslice(code);
        }
        Some(out)
    }

    fn decompress(&self, compressed: &BitVec) -> Option<Vec<u8>> {
        let mut output = vec![];

        let mut node = &self.root;
        assert!(!matches!(&self.root, &Node::Leaf { .. }));

        for bit in compressed {
            let Node::Branch { children } = node else {
                unreachable!();
            };
            node = &children[*bit as usize];

            if let Node::Leaf { symbol } = node {
                output.push(*symbol);
                node = &self.root;
            }
        }

        if ptr::eq(node, &self.root) {
            Some(output)
        } else {
            None
        }
    }
}
