module dut(input clk);

reg [`WIDTH-1:0] cnt;

always @(posedge clk) begin
cnt <= cnt + 1;
end

endmodule
