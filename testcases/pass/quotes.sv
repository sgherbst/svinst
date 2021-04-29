module t1 #(parameter P="TRUE")();
reg a=0;
b #(.P(P))b1(.a(a));
endmodule
