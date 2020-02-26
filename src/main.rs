use std::collections::HashMap;
use std::error::Error as StdError;
use std::fs::File;
use std::io::{Read, BufWriter, Write};
use std::path::PathBuf;
use std::{cmp, process};
use structopt::StructOpt;
use sv_parser::{parse_sv, SyntaxTree, unwrap_node, Locate, RefNode};
use sv_parser_error::Error;

#[derive(StructOpt)]
struct Opt {
    pub files: Vec<PathBuf>,

    /// Include path
    #[structopt(short = "i", long = "include", multiple = true, number_of_values = 1)]
    pub includes: Vec<PathBuf>,

    /// Output file
    #[structopt(short = "o", long = "output", default_value = "out.yaml")]
    pub out_file: String
}

fn main() {
    let opt = Opt::from_args();
    
    let mut defines = HashMap::new();
    let mut exit_code = 0;
    
    let write_file = File::create(opt.out_file).unwrap();
    let mut writer = BufWriter::new(&write_file);
    
    write!(&mut writer, "files:\n").unwrap();
    for path in &opt.files {
        match parse_sv(&path, &defines, &opt.includes, false) {
            Ok((syntax_tree, new_defines)) => {
				write!(&mut writer, "  - file_name: \"{}\"\n", path.to_str().unwrap()).unwrap();
				write!(&mut writer, "    mod_defs:\n").unwrap();
				analyze_mod_defs(&mut writer, &syntax_tree);
                defines = new_defines;
                println!("parse succeeded: {:?}", path);
            }
            Err(x) => {
                match x {
                    Error::Parse(Some((origin_path, origin_pos))) => {
                        println!("parse failed: {:?}", path);
                        print_parse_error(&origin_path, &origin_pos);
                    }
                    x => {
                        println!("parse failed: {:?} ({})", path, x);
                        let mut err = x.source();
                        while let Some(x) = err {
                            println!("  Caused by {}", x);
                            err = x.source();
                        }
                    }
                }
                exit_code = 1;
            }
        }
    }
    writer.flush().unwrap();

    // exit when done
    process::exit(exit_code);
}

static CHAR_CR: u8 = 0x0d;
static CHAR_LF: u8 = 0x0a;

fn print_parse_error(origin_path: &PathBuf, origin_pos: &usize) {
    let mut f = File::open(&origin_path).unwrap();
    let mut s = String::new();
    let _ = f.read_to_string(&mut s);

    let mut pos = 0;
    let mut column = 1;
    let mut last_lf = None;
    while pos < s.len() {
        if s.as_bytes()[pos] == CHAR_LF {
            column += 1;
            last_lf = Some(pos);
        }
        pos += 1;

        if *origin_pos == pos {
            let row = if let Some(last_lf) = last_lf {
                pos - last_lf
            } else {
                pos + 1
            };
            let mut next_crlf = pos;
            while next_crlf < s.len() {
                if s.as_bytes()[next_crlf] == CHAR_CR || s.as_bytes()[next_crlf] == CHAR_LF {
                    break;
                }
                next_crlf += 1;
            }

            let column_len = format!("{}", column).len();

            print!(" {}:{}:{}\n", origin_path.to_string_lossy(), column, row);

            print!("{}|\n", " ".repeat(column_len + 1));

            print!("{} |", column);

            let beg = if let Some(last_lf) = last_lf {
                last_lf + 1
            } else {
                0
            };
            print!(
                " {}\n",
                String::from_utf8_lossy(&s.as_bytes()[beg..next_crlf])
            );

            print!("{}|", " ".repeat(column_len + 1));

            print!(
                " {}{}\n",
                " ".repeat(pos - beg),
                "^".repeat(cmp::min(origin_pos + 1, next_crlf) - origin_pos)
            );
        }
    }
}

fn analyze_mod_defs<W: Write>(writer: &mut W, syntax_tree: &SyntaxTree) {
    // &SyntaxTree is iterable
    for node in syntax_tree {
        // The type of each node is RefNode
        match node {
            RefNode::ModuleDeclarationNonansi(x) => {
                // unwrap_node! gets the nearest ModuleIdentifier from x
                let id = unwrap_node!(x, ModuleIdentifier).unwrap();
                let id = get_identifier(id).unwrap();
                // Original string can be got by SyntaxTree::get_str(self, node: &RefNode)
                let id = syntax_tree.get_str(&id).unwrap();
                // Declare the new module
				write!(writer, "      - mod_name: \"{}\"\n", id).unwrap();
				write!(writer, "        mod_insts:\n").unwrap();
            }
            RefNode::ModuleDeclarationAnsi(x) => {
                let id = unwrap_node!(x, ModuleIdentifier).unwrap();
                let id = get_identifier(id).unwrap();
                let id = syntax_tree.get_str(&id).unwrap();
				write!(writer, "      - mod_name: \"{}\"\n", id).unwrap();
				write!(writer, "        mod_insts:\n").unwrap();
            }
            RefNode::ModuleInstantiation(x) => {
				// write the module name
				let id = unwrap_node!(x, ModuleIdentifier).unwrap();
				let id = get_identifier(id).unwrap();
                let id = syntax_tree.get_str(&id).unwrap();
                write!(writer, "          - mod_name: \"{}\"\n", id).unwrap();
                // write the instance name
				let id = unwrap_node!(x, InstanceIdentifier).unwrap();
				let id = get_identifier(id).unwrap();
                let id = syntax_tree.get_str(&id).unwrap();
                write!(writer, "            inst_name: \"{}\"\n", id).unwrap();
			}
            _ => (),
        }
    }
}

fn get_identifier(node: RefNode) -> Option<Locate> {
    // unwrap_node! can take multiple types
    match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier) {
        Some(RefNode::SimpleIdentifier(x)) => {
            return Some(x.nodes.0);
        }
        Some(RefNode::EscapedIdentifier(x)) => {
            return Some(x.nodes.0);
        }
        _ => None,
    }
}

