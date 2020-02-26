# svinst

This tool takes a SystemVerilog file as input and produces as output the module(s) declared in that file, along with the module(s) instantiated in each one of those module declarations.  It uses [sv-parser](https://github.com/dalance/sv-parser) and is adapted from [svlint](https://github.com/dalance/svlint).

## Installation

Download a binary from the [Releases](https://github.com/sgherbst/svinst/releases) tab, or clone and build using a ``make`` target for your system:

```shell
> git clone https://github.com/sgherbst/svinst.git
> cd svinst
> make release_lnx
> make release_win
> make release_mac
```

## Purpose

The Verilog language has contained features for defining configs and libraries for close to 20 years.  However, these features are not well-supported by open-source tools, and even some commercial synthesis tools.  By extracting a list of modules defined and instantiated in a file, a user can work around this problem by constructing their own design hierarchy outside of Verilog, and then passing that list of files back into the simulator / synthesis tool.

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
