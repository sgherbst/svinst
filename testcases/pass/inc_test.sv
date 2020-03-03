`include "inc.sv"
module inc_top;
    wire a, b, c;
    `MODULE_NAME I0 (
        .a(a),
        .b(b),
        .c(c)
    );
endmodule
