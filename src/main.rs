use std::collections::HashMap;
use std::error::Error as StdError;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::{cmp, process};
use structopt::StructOpt;
use sv_parser::{parse_sv, SyntaxTree, unwrap_node, Locate, RefNode, Define, DefineText};
use sv_parser_error;
use sv_parser_syntaxtree::*;
use enquote;

#[derive(StructOpt)]
struct Opt {
    pub files: Vec<PathBuf>,

    /// Define
    #[structopt(short = "d", long = "define", multiple = true, number_of_values = 1)]
    pub defines: Vec<String>,

    /// Include path
    #[structopt(short = "i", long = "include", multiple = true, number_of_values = 1)]
    pub includes: Vec<PathBuf>,

    /// Ignore any include
    #[structopt(long = "ignore-include")]
    pub ignore_include: bool,

    /// Show the full syntax tree rather than just module instantiation
    #[structopt(long = "full-tree")]
    pub full_tree: bool,

    /// Include whitespace in output syntax tree
    #[structopt(long = "include-whitespace")]
    pub include_whitespace: bool,
 
    /// Show the macro definitions after processing each file
    #[structopt(long = "show-macro-defs")]
    pub show_macro_defs: bool,

    /// Treat each file as completely separate, not updating define variables after each file
    #[structopt(long = "separate")]
    pub separate: bool,

    /// Allow incomplete
    #[structopt(long = "allow_incomplete")]
    pub allow_incomplete: bool
}

fn main() {
    let opt = Opt::from_args();
    let exit_code = run_opt(&opt);
    process::exit(exit_code);
}

fn run_opt(
	opt: &Opt
) -> i32 {

    // read in define variables
    let mut defines = HashMap::new();
    for define in &opt.defines {
		let mut define = define.splitn(2, '=');
        let ident = String::from(define.next().unwrap());
        let text = if let Some(x) = define.next() {
            let x = enquote::unescape(x, None).unwrap();
            Some(DefineText::new(x, None))
        } else {
            None
        };
        let define = Define::new(ident.clone(), vec![], text);
        defines.insert(ident, Some(define));
	}
    
    // flag to determine parsing status
    let mut exit_code = 0;
    
    // parse files
    println!("files:");
    for path in &opt.files {
        match parse_sv(&path, &defines, &opt.includes, opt.ignore_include, opt.allow_incomplete) {
            Ok((syntax_tree, new_defines)) => {
				println!("  - file_name: {}", escape_str(path.to_str().unwrap()));
				if !opt.full_tree {
					println!("    defs:");
					analyze_defs(&syntax_tree);
				} else {
					println!("    syntax_tree:");
					print_full_tree(&syntax_tree, opt.include_whitespace);
				}
				// update the preprocessor state if desired
				if !opt.separate {
					defines = new_defines;
				}
				// show macro definitions if desired
				if opt.show_macro_defs {
					println!("    macro_defs:");
					show_macro_defs(&defines);
				}
            }
            Err(x) => {
                match x {
                    sv_parser_error::Error::Parse(Some((origin_path, origin_pos))) => {
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
    
    // return exit code
    exit_code
}

static CHAR_CR: u8 = 0x0d;
static CHAR_LF: u8 = 0x0a;

fn print_parse_error(
	origin_path: &PathBuf,
	origin_pos: &usize
) {
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

fn show_macro_defs(
	defines: &HashMap<String, Option<Define>>
) {
	for (_, value) in defines.into_iter() {
		match value {
			Some(define) => println!("      - '{:?}'", define),
			_ => (),
		}
	}
}

fn analyze_defs(
	syntax_tree: &SyntaxTree
) {
    // &SyntaxTree is iterable
    for node in syntax_tree {
        // The type of each node is RefNode
        match node {
            RefNode::ModuleDeclarationNonansi(x) => {
                // unwrap_node! gets the nearest ModuleIdentifier from x
				let id = match unwrap_node!(x, ModuleIdentifier) {
					None => { continue; },
					Some(x) => x
				};
				let id = match get_identifier(id) {
					None => { continue; },
					Some(x) => x
				};				
                // Original string can be got by SyntaxTree::get_str(self, node: &RefNode)
                let id = match syntax_tree.get_str(&id) {
					None => { continue; },
					Some(x) => x
				};	
                // Declare the new module
				println!("      - mod_name: {}", escape_str(id));
				println!("        insts:");
            }
            RefNode::ModuleDeclarationAnsi(x) => {
				let id = match unwrap_node!(x, ModuleIdentifier) {
					None => { continue; },
					Some(x) => x
				};
				let id = match get_identifier(id) {
					None => { continue; },
					Some(x) => x
				};		
                let id = match syntax_tree.get_str(&id) {
					None => { continue; },
					Some(x) => x
				};	
				println!("      - mod_name: {}", escape_str(id));
				println!("        insts:");
            }
            RefNode::PackageDeclaration(x) => {
				let id = match unwrap_node!(x, PackageIdentifier) {
					None => { continue; },
					Some(x) => x
				};
				let id = match get_identifier(id) {
					None => { continue; },
					Some(x) => x
				};		
                let id = match syntax_tree.get_str(&id) {
					None => { continue; },
					Some(x) => x
				};	
				println!("      - pkg_name: {}", escape_str(id));
				println!("        insts:");
            }
            RefNode::InterfaceDeclaration(x) => {
				let id = match unwrap_node!(x, InterfaceIdentifier) {
					None => { continue; },
					Some(x) => x
				};
				let id = match get_identifier(id) {
					None => { continue; },
					Some(x) => x
				};		
                let id = match syntax_tree.get_str(&id) {
					None => { continue; },
					Some(x) => x
				};
				println!("      - intf_name: {}", escape_str(id));
				println!("        insts:");
            }
            RefNode::ModuleInstantiation(x) => {
				// write the module name
				let id = match unwrap_node!(x, ModuleIdentifier) {
					None => { continue; },
					Some(x) => x
				};
				let id = match get_identifier(id) {
					None => { continue; },
					Some(x) => x
				};		
                let id = match syntax_tree.get_str(&id) {
					None => { continue; },
					Some(x) => x
				};
                println!("          - mod_name: {}", escape_str(id));
                // write the instance name
				let id = match unwrap_node!(x, InstanceIdentifier) {
					None => { continue; },
					Some(x) => x
				};
				let id = match get_identifier(id) {
					None => { continue; },
					Some(x) => x
				};		
                let id = match syntax_tree.get_str(&id) {
					None => { continue; },
					Some(x) => x
				};
                println!("            inst_name: {}", escape_str(id));
			}
            RefNode::PackageImportItem(x) => {
				// write the package name
				let id = match unwrap_node!(x, PackageIdentifier) {
					None => { continue; },
					Some(x) => x
				};
				let id = match get_identifier(id) {
					None => { continue; },
					Some(x) => x
				};		
                let id = match syntax_tree.get_str(&id) {
					None => { continue; },
					Some(x) => x
				};
                println!("          - pkg_name: {}", escape_str(id));
			}
			RefNode::ImplicitClassHandleOrClassScope(x) => {
				// write the package name
				let id = match unwrap_node!(x, ClassIdentifier) {
					None => { continue; },
					Some(x) => x
				};
				let id = match get_identifier(id) {
					None => { continue; },
					Some(x) => x
				};		
                let id = match syntax_tree.get_str(&id) {
					None => { continue; },
					Some(x) => x
				};
                println!("          - pkg_name: {}", escape_str(id));
			}
			RefNode::ImplicitClassHandleOrClassScopeOrPackageScope(x) => {
				// write the package name
				let id = match unwrap_node!(x, ClassIdentifier) {
					None => { continue; },
					Some(x) => x
				};
				let id = match get_identifier(id) {
					None => { continue; },
					Some(x) => x
				};
                let id = match syntax_tree.get_str(&id) {
					None => { continue; },
					Some(x) => x
				};
                println!("          - pkg_name: {}", escape_str(id));
			}
            _ => (),
        }
    }
}

fn print_full_tree(
	syntax_tree: &SyntaxTree,
	include_whitespace: bool
) {
	let mut skip = false;
	let mut depth = 3;
	for node in syntax_tree.into_iter().event() {
		match node {
			NodeEvent::Enter(RefNode::Locate(locate)) => {
				if !skip {
					println!("{}- Token: {}",
					         "  ".repeat(depth),
					         escape_str(syntax_tree.get_str(locate).unwrap()));
					println!("{}  Line: {}",
					         "  ".repeat(depth),
					         locate.line);
				}
				depth += 1;
			}
			NodeEvent::Enter(RefNode::WhiteSpace(_)) => {
				if !include_whitespace {
					skip = true;
				}
			}
			NodeEvent::Leave(RefNode::WhiteSpace(_)) => {
				skip = false;
			}
			NodeEvent::Enter(x) => {
				if !skip {
					println!("{}- {}:",
					         "  ".repeat(depth),
					         x);
				}
				depth += 1;
			}
			NodeEvent::Leave(_) => {
				depth -= 1;
			}
		}
	}
}

fn get_identifier(
	node: RefNode
) -> Option<Locate> {
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

// escape_str adapted from this code:
// https://github.com/chyh1990/yaml-rust/blob/6cd3ce4abe6894443645c48bdc375808ec911493/src/emitter.rs#L43-L104
fn escape_str(v: &str) -> String {
    let mut wr = String::new();
    
    wr.push_str("\"");

    let mut start = 0;

    for (i, byte) in v.bytes().enumerate() {
        let escaped = match byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\x00' => "\\u0000",
            b'\x01' => "\\u0001",
            b'\x02' => "\\u0002",
            b'\x03' => "\\u0003",
            b'\x04' => "\\u0004",
            b'\x05' => "\\u0005",
            b'\x06' => "\\u0006",
            b'\x07' => "\\u0007",
            b'\x08' => "\\b",
            b'\t' => "\\t",
            b'\n' => "\\n",
            b'\x0b' => "\\u000b",
            b'\x0c' => "\\f",
            b'\r' => "\\r",
            b'\x0e' => "\\u000e",
            b'\x0f' => "\\u000f",
            b'\x10' => "\\u0010",
            b'\x11' => "\\u0011",
            b'\x12' => "\\u0012",
            b'\x13' => "\\u0013",
            b'\x14' => "\\u0014",
            b'\x15' => "\\u0015",
            b'\x16' => "\\u0016",
            b'\x17' => "\\u0017",
            b'\x18' => "\\u0018",
            b'\x19' => "\\u0019",
            b'\x1a' => "\\u001a",
            b'\x1b' => "\\u001b",
            b'\x1c' => "\\u001c",
            b'\x1d' => "\\u001d",
            b'\x1e' => "\\u001e",
            b'\x1f' => "\\u001f",
            b'\x7f' => "\\u007f",
            _ => continue,
        };

        if start < i {
            wr.push_str(&v[start..i]);
        }

        wr.push_str(escaped);

        start = i + 1;
    }

    if start != v.len() {
        wr.push_str(&v[start..]);
    }

    wr.push_str("\"");
    
    wr
}

#[cfg(test)]
mod tests {
	use super::*;

	fn run_opt_expect(opt: &Opt, val: i32) {
		let ret = run_opt(&opt);
		assert_eq!(ret, val);
	}
	 
    fn expect_pass(opt: &Opt) {
		run_opt_expect(opt, 0);
	}
	
	fn expect_fail(opt: &Opt) {
		run_opt_expect(opt, 1);
	}
    
    #[test]
    fn test_test() {
        let opt = Opt{
			files: vec![PathBuf::from("testcases/pass/test.sv")],
			defines: vec![],
			includes: vec![],
			full_tree: false,
			include_whitespace: false,
			ignore_include: false,
			separate: false,
			show_macro_defs: false,
			allow_incomplete: false
		};
		expect_pass(&opt);
    }
    
    #[test]
    fn test_broken() {
        let opt = Opt{
			files: vec![PathBuf::from("testcases/fail/broken.sv")],
			defines: vec![],
			includes: vec![],
			full_tree: false,
			include_whitespace: false,
			ignore_include: false,
			separate: false,
			show_macro_defs: false,
			allow_incomplete: false
		};
		expect_fail(&opt);
    }
    
    #[test]
    fn test_inc_test() {
        let opt = Opt{
			files: vec![PathBuf::from("testcases/pass/inc_test.sv")],
			defines: vec![],
			includes: vec![PathBuf::from("testcases/pass")],
			full_tree: false,
			include_whitespace: false,
			ignore_include: false,
			separate: false,
			show_macro_defs: false,
			allow_incomplete: false
		};
		expect_pass(&opt);
    }
    
    #[test]
    fn test_def_test() {
        let opt = Opt{
			files: vec![PathBuf::from("testcases/pass/def_test.sv")],
			defines: vec![String::from("MODULE_NAME=module_name_from_define"),
			              String::from("EXTRA_INSTANCE")],
			includes: vec![],
			full_tree: false,
			include_whitespace: false,
			ignore_include: false,
			separate: false,
			show_macro_defs: true,
			allow_incomplete: false
		};
		expect_pass(&opt);
    }
 
	#[test]
    fn test_simple() {
        let opt = Opt{
			files: vec![PathBuf::from("testcases/pass/simple.sv")],
			defines: vec![],
			includes: vec![],
			full_tree: true,
			include_whitespace: false,
			ignore_include: false,
			separate: false,
			show_macro_defs: false,
			allow_incomplete: false
		};
		expect_pass(&opt);
    }
    
    #[test]
    fn test_quotes() {
        let opt = Opt{
			files: vec![PathBuf::from("testcases/pass/quotes.sv")],
			defines: vec![],
			includes: vec![],
			full_tree: true,
			include_whitespace: false,
			ignore_include: false,
			separate: false,
			show_macro_defs: false,
			allow_incomplete: false
		};
		expect_pass(&opt);
    }
    
    #[test]
    fn test_pkg() {
        let opt = Opt{
			files: vec![PathBuf::from("testcases/pass/pkg.sv")],
			defines: vec![],
			includes: vec![],
			full_tree: false,
			include_whitespace: false,
			ignore_include: false,
			separate: false,
			show_macro_defs: false,
			allow_incomplete: false
		};
		expect_pass(&opt);
    }
    
    #[test]
    fn test_intf() {
        let opt = Opt{
			files: vec![PathBuf::from("testcases/pass/intf.sv")],
			defines: vec![],
			includes: vec![],
			full_tree: false,
			include_whitespace: false,
			ignore_include: false,
			separate: false,
			show_macro_defs: false,
			allow_incomplete: false
		};
		expect_pass(&opt);
    }
    
    #[test]
    fn test_class() {
        let opt = Opt{
			files: vec![PathBuf::from("testcases/pass/class.sv")],
			defines: vec![],
			includes: vec![],
			full_tree: false,
			include_whitespace: false,
			ignore_include: false,
			separate: false,
			show_macro_defs: false,
			allow_incomplete: false
		};
		expect_pass(&opt);
    }

    #[test]
    fn test_multi() {
        let opt = Opt{
			files: vec![
			    PathBuf::from("testcases/pass/multi/define1.v"),
			    PathBuf::from("testcases/pass/multi/test1.sv"),
			    PathBuf::from("testcases/pass/multi/define2.v"),
			    PathBuf::from("testcases/pass/multi/dut.v")
			],
			defines: vec![],
			includes: vec![],
			full_tree: false,
			include_whitespace: false,
			ignore_include: false,
			separate: false,
			show_macro_defs: false,
			allow_incomplete: false
		};
		expect_pass(&opt);
    }
}
