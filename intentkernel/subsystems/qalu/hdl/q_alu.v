`timescale 1ns/1ps

module q_alu (
    input  wire [1:0] a,
    input  wire [1:0] b,
    input  wire [3:0] op,
    output reg  [1:0] y
);
    always @* begin
        case (op)
            4'h0: y = a + b;
            4'h1: y = a - b;
            4'h2: y = (a < b) ? a : b;
            4'h3: y = (a > b) ? a : b;
            4'h4: y = (a + b) & 2'b11;
            4'h5: y = a & b;
            4'h6: y = a | b;
            4'h7: y = a ^ b;
            4'h8: y = (a * b) & 2'b11;
            default: y = 2'b00;
        endcase
    end
endmodule

module q_alu_tb;
    reg [1:0] a;
    reg [1:0] b;
    reg [3:0] op;
    wire [1:0] y;

    q_alu dut (
        .a(a),
        .b(b),
        .op(op),
        .y(y)
    );

    initial begin
        a = 2'd0; b = 2'd0; op = 4'h0; #1;
        a = 2'd2; b = 2'd3; op = 4'h0; #1;
        a = 2'd3; b = 2'd1; op = 4'h1; #1;
        a = 2'd1; b = 2'd2; op = 4'h2; #1;
        a = 2'd1; b = 2'd2; op = 4'h3; #1;
        a = 2'd3; b = 2'd3; op = 4'h8; #1;
        $display("Q-ALU testbench complete");
    end
endmodule
