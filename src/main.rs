use std::cmp::min;
use std::os::raw::{c_char, c_void};
use std::rc::Rc;

use crate::HaffmanNode::{Leaf, Node};

fn main() {
    make_image();
    let row = fetch_row();
    let slice = &row[..];

    println!("Source row: {:?}\n", slice);

    let alphabet = make_alphabet(slice);

    println!("Source alphabet: {:?}", alphabet.as_slice());
    println!("Alphabet length: {:?}", alphabet.len());
    println!("Entropy: {:?}\n", calc_entropy(alphabet.as_slice()));
    let simple_dict = make_simple_dict(alphabet.as_slice());
    println!("Average minimal binary code length: {}\n", average_min_binary_code(alphabet.as_slice()));

    println!("Shennon tree representation: ");
    let mut shennon_dict = vec![String::new(); alphabet.len()];
    build_shennon_tree(
        alphabet.as_slice(),
        0,
        alphabet.len() - 1,
        "",
        "  ",
        shennon_dict.as_mut_slice(),
    );
    println!("Shennon tree end\n");
    println!("Summary Shennon dictionary");
    print_dict(shennon_dict.as_slice(), alphabet.as_slice());
    println!();

    let haffman_dict = make_haffman_dict(alphabet.as_slice());
    println!("\nSummary Haffman dictionary");
    print_dict(haffman_dict.as_slice(), alphabet.as_slice());
    println!();
    print!("Shennon encoded sequence: ");
    let shenon_len = encode_sequence(&row[..], shennon_dict.as_slice(), alphabet.as_slice());
    println!("Shennon length: {}\n", shenon_len);

    print!("Haffman encoded sequence: ");
    let haffman_len = encode_sequence(&row[..], haffman_dict.as_slice(), alphabet.as_slice());
    println!("Haffman length: {}\n", haffman_len);

    print!("Simple encoded sequence: ");
    let simple_len = encode_sequence(&row[..], simple_dict.as_slice(), alphabet.as_slice());
    println!("Simple length: {}\n", simple_len);
}

fn encode_sequence(sequence: &[u8], dictionary: &[String], alphabet: &[(u8, u8)]) -> u32 {
    let mut counter = 0u32;
    for x in sequence {
        let index = lookup_index(alphabet, *x);
        let code = dictionary[index].as_str();

        print!("{}", code);
        counter += code.len() as u32
    }
    println!();

    counter
}

fn lookup_index(alphabet: &[(u8,u8)], value: u8) -> usize{
    for i in 0..alphabet.len() {
        let e = alphabet[i].0;
        if e == value { return i;}
    }
    panic!()
}

#[derive(Clone, Debug)]
enum HaffmanNode {
    Node(u32, Rc<HaffmanNode>, Rc<HaffmanNode>),
    Leaf(u32, usize),
}
fn make_haffman_dict(alphabet: &[(u8, u8)]) -> Vec<String> {
    let node = {
        let mut nodes = Vec::<Rc<HaffmanNode>>::new();
        for i in 0..alphabet.len() {
            let (p, e) = alphabet[i];
            nodes.push(
                Rc::new(
                    Leaf(e as u32, i)
                )
            );
        }
        make_haffman_tree(&mut nodes)
    };

    let mut result = vec![String::new(); alphabet.len()];
    println!("Haffman tree:");
    do_haffman_dict(node, alphabet, "", "", result.as_mut_slice());
    result
}

fn do_haffman_dict(node: Rc<HaffmanNode>, alphabet: &[(u8, u8)], current_prefix: &str, current_indent: &str, result: &mut [String]) {
    if let Leaf(_, index) = *node {
        println!("{} -V ({}) -- {}", current_indent, alphabet[index].0, current_prefix);
        result[index] = current_prefix.to_string();
        return;
    }
    if let Node(weight, left, right) = &*node {
        println!("{} -N({}) {}", current_indent, weight, current_prefix);
        let new_indent = format!("{}  ", current_indent);
        let new_indent = new_indent.as_str();

        let new_prefix = format!("{}0", current_prefix);
        let new_prefix = new_prefix.as_str();
        do_haffman_dict(left.clone(), alphabet, new_prefix, new_indent, result);

        let new_prefix = format!("{}1", current_prefix);
        let new_prefix = new_prefix.as_str();
        do_haffman_dict(right.clone(), alphabet, new_prefix, new_indent, result);
    }
}

fn make_haffman_tree(nodes: &mut Vec<Rc<HaffmanNode>>) -> Rc<HaffmanNode> {
    if nodes.len() == 1 { return nodes.get(0).unwrap().clone(); }
    let mut min1 = 0u32;
    let mut min2 = 0u32;
    let mut min1i = 0usize;
    let mut min2i = 0usize;

    for i in 0..nodes.len() {
        let weight = match **(nodes.get(i).unwrap()) {
            Leaf(w, _) => w,
            Node(w, _, _) => w,
        };
        if min1 == 0 {
            min1 = weight;
            min1i = i;
            continue;
        } else if min2 == 0 {
            min2 = weight;
            min2i = i;
            continue;
        }

        if weight < min2 {
            min1 = min2;
            min1i = min2i;
            min2 = weight;
            min2i = i;
        } else if weight < min1 {
            min1 = weight;
            min1i = i;
        }
    }

    let first = nodes[min1i].clone();
    let second = nodes[min2i].clone();

    nodes.remove(min1i);
    if min2i > min1i {
        nodes.remove(min2i - 1);
    } else {
        nodes.remove(min2i);
    }

    let w1: u32;
    let w2: u32;

    match *first {
        Leaf(weight, _) => w1 = weight,
        Node(weight, _, _) => w1 = weight
    }
    match *second {
        Leaf(weight, _) => w2 = weight,
        Node(weight, _, _) => w2 = weight
    }

    nodes.push(Rc::new(
        Node(w1 + w2, first, second)
    ));

    make_haffman_tree(nodes)
}


fn print_dict(dict: &[String], alphabet: &[(u8, u8)]) {
    for i in 0..dict.len() {
        println!("  {} == {}", alphabet[i].0, dict[i])
    }
}

fn build_shennon_tree(
    alphabet: &[(u8, u8)],
    l: usize,
    r: usize,
    current_prefix: &str,
    current_indent: &str,
    result: &mut [String],
) {
    if l == r {
        println!("{} -V: ({}) -- {}", current_indent, alphabet[l].0, current_prefix);
        result[l] = current_prefix.to_string();
        return;
    }
    if l > r { return; }
    let m = (l + r) / 2;

    let new_indent = format!("{} ", current_indent);
    let new_indent = new_indent.as_str();

    let new_prefix = format!("{}0", current_prefix);
    let new_prefix = new_prefix.as_str();

    println!("{} -N: {}", current_indent, new_prefix);
    build_shennon_tree(alphabet, l, m, new_prefix, new_indent, result);

    let new_prefix = format!("{}1", current_prefix);
    let new_prefix = new_prefix.as_str();
    println!("{} -N: {}", current_indent, new_prefix);
    build_shennon_tree(alphabet, m + 1, r, new_prefix, new_indent, result)

}

fn make_simple_dict(alphabet: &[(u8, u8)]) -> Vec<String> {
    println!("Binary codes:");
    let len = calc_minimum_binary_code(alphabet.len() as u8);
    let mut result = Vec::<String>::new();
    for i in 0..alphabet.len() {
        let element = alphabet[i];

        let code = format!("{i:>0len$b}", i = i, len = len as usize);
        println!("    {} -- {element:?}",code.as_str(), element = element);
        result.push(code);
    }

    result
}

fn average_min_binary_code(alphabet: &[(u8, u8)]) -> f32 {
    let mut sum = 0u32;
    for i in 0..alphabet.len() {
        sum += calc_minimum_binary_code(i as u8);
    }

    return sum as f32 / alphabet.len() as f32;
}

fn calc_minimum_binary_code(value: u8) -> u32 {
    if value & 0b1000_0000 != 0 { return 8; }
    if value & 0b0100_0000 != 0 { return 7; }
    if value & 0b0010_0000 != 0 { return 6; }
    if value & 0b0001_0000 != 0 { return 5; }
    if value & 0b0000_1000 != 0 { return 4; }
    if value & 0b0000_0100 != 0 { return 3; }
    if value & 0b0000_0010 != 0 { return 2; }
    return 1;
}

fn calc_entropy(alphabet: &[(u8, u8)]) -> f32 {
    let mut result = 0f32;
    for (val, count) in alphabet {
        let probability = *count as f32 / 128.0;

        result += probability * probability.log2()
    }

    -result
}

fn make_alphabet(src: &[u8]) -> Vec<(u8, u8)> {
    let mut counter = [0u8; 256];
    for x in src {
        counter[*x as usize] += 1;
    }

    let mut result = Vec::<(u8, u8)>::new();
    for e in 0..counter.len() {
        if counter[e] > 0 {
            result.push((e as u8, counter[e]))
        }
    }

    result
}

fn make_image() {
    unsafe {
        let bmp = BMP_Create(128, 128, 8);
        for i in 0..255 {
            BMP_SetPaletteColor(bmp, i,  i as u8, i as u8, i as u8);
        }
        for x in 0..128 {
            for y in 0..128 {
                BMP_SetPixelIndex(bmp, x, y, (x + y) as u8);
            }
        }
        BMP_WriteFile(bmp, "image.bmp\0".as_ptr());
    }
}

fn fetch_row() -> [u8; 128] {
    unsafe {
        let args = std::env::args().collect::<Vec<String>>();
        let name = format!("{}\0", args[1].as_str());
        let name = name.as_str();

        let bmp = BMP_ReadFile(name.as_ptr());

        let width = BMP_GetDepth(bmp);
        if width != 8 {
            panic!("ALERT! ALERT! ALERT! {}", width)
        }
        if BMP_GetHeight(bmp) != 128 || BMP_GetWidth(bmp) != 128 {
            panic!("ALERT! ALERT! ALERT!")
        }

        let mut result = [0u8; 128];

        for x in 0..128 {
            let mut value = 0u8;

            BMP_GetPixelIndex(bmp, x, 64, &mut value);
            result[x as usize] = min((value as f32 / 20.0).round() as u32 * 20, 240) as u8;
        }

        BMP_Free(bmp);
        quntalize_image(name);
        result
    }
}

unsafe fn quntalize_image(name: &str) {
    let source = BMP_ReadFile(name.as_ptr());
    let target = BMP_Create(128, 128, 8);

    for i in 0..=255 {
        let mut r = 0u8;
        let mut g = 0u8;
        let mut b = 0u8;

        BMP_GetPaletteColor(source, i, &mut r, &mut g, &mut b);
        BMP_SetPaletteColor(target, i, r, g, b);
    }

    for x in 0..BMP_GetWidth(source) {
        for y in 0..BMP_GetHeight(source) {
            let mut value = 0u8;
            BMP_GetPixelIndex(source, x, y, &mut value);
            BMP_SetPixelIndex(target, x, y, min((value as f32 / 20.0).round() as u32 * 20, 240) as u8)
        }
    }

    let name = "quant.bmp";
    BMP_WriteFile(target, name.as_ptr());
    BMP_Free(source);
    BMP_Free(target);

}

type BMP = c_void;
type PBMP = *const BMP;

extern "C" {
    fn BMP_ReadFile(name: *const u8) -> *const BMP;
    fn BMP_WriteFile(bmp: PBMP, name: *const u8);

    fn BMP_GetWidth(bmp: PBMP) -> u32;
    fn BMP_GetHeight(bmp: PBMP) -> u32;

    fn BMP_GetDepth(bmp: PBMP) -> u16;

    fn BMP_GetPixelIndex(bmp: PBMP, x: u32, y: u32, value: *mut u8);
    fn BMP_SetPixelIndex(bmp: PBMP, x: u32, y: u32, value: u8);

    fn BMP_SetPaletteColor(bmp: PBMP, index: u8, r: u8, g: u8, b: u8);
    fn BMP_GetPaletteColor(bmp: PBMP, index: u8, r: &mut u8, g: &mut u8, b: &mut u8);

    fn BMP_Create(width: u32, height: u32, depth: u16) -> PBMP;
    fn BMP_Free(bmp: PBMP);
}