module test;

reg [`WIDTH-1:0] cnt;
reg clk=0;
always #1 clk = ~clk;

dut u0(clk);

endmodule
