# svinst

This tool takes a SystemVerilog file as input and produces as output the module(s) declared in that file, along with the module(s) instantiated in each one of those module declarations.  It uses [sv-parser](https://github.com/dalance/sv-parser) and is adapted from [svlint](https://github.com/dalance/svlint).

## Purpose

The Verilog language has contained features for defining configs and libraries for close to 20 years.  However, these features are not well-supported by open-source tools, and even some commercial synthesis tools.  By extracting a list of modules defined and instantiated in a file, a user can work around this problem by constructing their own design hierarchy outside of Verilog, and then passing that list of files back into the simulator / synthesis tool.
