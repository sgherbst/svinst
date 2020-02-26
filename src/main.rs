use std::collections::HashMap;
use std::error::Error as StdError;
use std::fs::File;
use std::io::Read;
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
}

fn main() {
    let opt = Opt::from_args();
    
    let mut defines = HashMap::new();
    let mut exit_code = 0;
        
    println!("files:");
    for path in &opt.files {
        match parse_sv(&path, &defines, &opt.includes, false) {
            Ok((syntax_tree, new_defines)) => {
				println!("  - file_name: \"{}\"", path.to_str().unwrap());
				println!("    mod_defs:");
				analyze_mod_defs(&syntax_tree);
                defines = new_defines;
            }
            Err(x) => {
                match x {
                    Error::Parse(Some((origin_path, origin_pos))) => {
                        eprintln!("parse failed: {:?}", path);
                        print_parse_error(&origin_path, &origin_pos);
                    }
                    x => {
                        eprintln!("parse failed: {:?} ({})", path, x);
                        let mut err = x.source();
                        while let Some(x) = err {
                            eprintln!("  Caused by {}", x);
                            err = x.source();
                        }
                    }
                }
                exit_code = 1;
            }
        }
    }

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

            eprint!(" {}:{}:{}\n", origin_path.to_string_lossy(), column, row);

            eprint!("{}|\n", " ".repeat(column_len + 1));

            eprint!("{} |", column);

            let beg = if let Some(last_lf) = last_lf {
                last_lf + 1
            } else {
                0
            };
            eprint!(
                " {}\n",
                String::from_utf8_lossy(&s.as_bytes()[beg..next_crlf])
            );

            eprint!("{}|", " ".repeat(column_len + 1));

            eprint!(
                " {}{}\n",
                " ".repeat(pos - beg),
                "^".repeat(cmp::min(origin_pos + 1, next_crlf) - origin_pos)
            );
        }
    }
}

fn analyze_mod_defs(syntax_tree: &SyntaxTree) {
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
				println!("      - mod_name: \"{}\"", id);
				println!("        mod_insts:");
            }
            RefNode::ModuleDeclarationAnsi(x) => {
                let id = unwrap_node!(x, ModuleIdentifier).unwrap();
                let id = get_identifier(id).unwrap();
                let id = syntax_tree.get_str(&id).unwrap();
				println!("      - mod_name: \"{}\"", id);
				println!("        mod_insts:");
            }
            RefNode::ModuleInstantiation(x) => {
				// write the module name
				let id = unwrap_node!(x, ModuleIdentifier).unwrap();
				let id = get_identifier(id).unwrap();
                let id = syntax_tree.get_str(&id).unwrap();
                println!("          - mod_name: \"{}\"", id);
                // write the instance name
				let id = unwrap_node!(x, InstanceIdentifier).unwrap();
				let id = get_identifier(id).unwrap();
                let id = syntax_tree.get_str(&id).unwrap();
                println!("            inst_name: \"{}\"", id);
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

