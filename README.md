# svinst

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Actions Status](https://github.com/sgherbst/svinst/workflows/Regression/badge.svg)](https://github.com/sgherbst/svinst/actions)
[![codecov](https://codecov.io/gh/sgherbst/svinst/branch/master/graph/badge.svg)](https://codecov.io/gh/sgherbst/svinst)
[![Crates.io](https://img.shields.io/crates/v/svinst.svg)](https://crates.io/crates/svinst)

This tool takes a SystemVerilog file as input and produces as output the module(s) declared in that file, along with the module(s) instantiated in each one of those module declarations.  It uses [sv-parser](https://github.com/dalance/sv-parser) and is adapted from [svlint](https://github.com/dalance/svlint).

## Purpose

The Verilog language has contains features for defining configs and libraries.  However, these features are not well-supported by open-source tools, and even some commercial synthesis tools.  By extracting a list of modules defined and instantiated in a file, a user can work around this problem by constructing their own design hierarchy outside of Verilog, and then passing that list of files back into the simulator / synthesis tool.

## Installation

Download a binary for your system from the [Releases](https://github.com/sgherbst/svinst/releases) tab.

If you want to build the code yourself, you'll need to have [Rust](https://www.rust-lang.org/tools/install) installed.  Then clone+build using the appropriate Makefile target. 

```shell
> git clone https://github.com/sgherbst/svinst.git
> cd svinst
> make release_lnx
> make release_win
> make release_mac
```

## Usage

The ``svinst`` binary accepts one or more SystemVerilog files as input, and prints a YAML-formatted representation of the modules defined and instantiated in those files:

```shell
> svinst verilog/test.sv
files:
  - file_name: "verilog/test.sv"
    mod_defs:
      - mod_name: "A"
        mod_insts:
      - mod_name: "B"
        mod_insts:
      - mod_name: "C"
        mod_insts:
          - mod_name: "A"
            inst_name: "I0"
          - mod_name: "B"
            inst_name: "I1"
      - mod_name: "D"
        mod_insts:
          - mod_name: "X"
            inst_name: "I0"
          - mod_name: "Y"
            inst_name: "I1"
```

If there are any parsing errors, the return code of ``svinst`` is nonzero, and the error message(s) will be sent to ``stderr``:

```shell
> svinst verilog/broken.sv > /dev/null
parse failed: "verilog/broken.sv"
 verilog/broken.sv:5:10
  |
5 | endmodule
  |
> echo $?
1
```

It is also possible to specify files to be included on the command line, via the ``-i INCLUDE_PATH`` option.  Multiple include paths may be specified; pass each separately via individual ``-i`` options.

```shell
> svinst verilog/inc_test.sv -i verilog/
files:
  - file_name: "verilog/inc_test.sv"
    mod_defs:
      - mod_name: "inc_top"
        mod_insts:
          - mod_name: "mod_name_from_inc_sv"
            inst_name: "I0"
```

Pre-processor defines can be set from the command line as well.  In this example, the first ``define`` has both a name and a value, controlling the name of the instantiated module from a ``define`` variable.  The second define has only a name, and it causes a second module to be instantiated only if it has be defined.

```shell
> svinst verilog/def_test.sv -d MODULE_NAME=module_name_from_define -d EXTRA_INSTANCE
files:
  - file_name: "verilog/def_test.sv"
    mod_defs:
      - mod_name: "def_top"
        mod_insts:
          - mod_name: "module_name_from_define"
            inst_name: "I0"
          - mod_name: "module_from_ifdef"
            inst_name: "I1"
```

It is also possible to generate the full syntax tree for SystemVerilog file(s) using the ``full-tree`` option.  The output is still in YAML format:

```shell
> svinst verilog/simple.sv --full-tree
files:
  - file_name: "verilog/simple.sv"
    syntax_tree:
      - SourceText:
        - Description:
          - ModuleDeclaration:
            - ModuleDeclarationAnsi:
              - ModuleAnsiHeader:
                - ModuleKeyword:
                  - Keyword:
                    - Token: "module"
                      Line: 1
                - ModuleIdentifier:
                  - Identifier:
                    - SimpleIdentifier:
                      - Token: "A"
                        Line: 1
                - Symbol:
                  - Token: ";"
                    Line: 1
              - Keyword:
                - Token: "endmodule"
                  Line: 2
```
