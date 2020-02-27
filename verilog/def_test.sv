module def_top;
    wire a, b, c;
    `MODULE_NAME I0 (
        .a(a),
        .b(b),
        .c(c)
    );

    `ifdef EXTRA_INSTANCE
        wire d, e, f;
        module_from_ifdef I1 (
            .d(d),
            .e(e),
            .f(f)
        );
    `endif
endmodule
