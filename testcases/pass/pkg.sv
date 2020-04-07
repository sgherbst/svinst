package i;
endpackage

module A import b::*; #(
) (
);
endmodule

package j;
endpackage

module B import c::d; #(
) (
);
endmodule

module E;
    import f::*;
    import g::h;
endmodule

package k;
endpackage
