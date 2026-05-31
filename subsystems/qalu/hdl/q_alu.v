`timescale 1ns / 1ps

module q_alu #(
    parameter QITS = 8
) (
    input  wire [2*QITS-1:0] a,
    input  wire [2*QITS-1:0] b,
    input  wire [3:0]         op,
    output reg  [2*QITS-1:0] result
);

localparam WIDTH = 2 * QITS;

function [1:0] qit_min(input [1:0] x, input [1:0] y);
    qit_min = (x < y) ? x : y;
endfunction

function [1:0] qit_max(input [1:0] x, input [1:0] y);
    qit_max = (x > y) ? x : y;
endfunction

integer i;
reg [WIDTH-1:0] a_val;
reg [WIDTH-1:0] b_val;
reg [WIDTH-1:0] tmp;

always @* begin
    a_val = a;
    b_val = b;
    case (op)
        4'd0: result = a_val + b_val;
        4'd1: result = a_val - b_val;
        4'd2: result = a_val * b_val;
        4'd3: begin
            tmp = 0;
            for (i = 0; i < QITS; i = i + 1) begin
                tmp[2*i +: 2] = qit_min(a[2*i +: 2], b[2*i +: 2]);
            end
            result = tmp;
        end
        4'd4: begin
            tmp = 0;
            for (i = 0; i < QITS; i = i + 1) begin
                tmp[2*i +: 2] = qit_max(a[2*i +: 2], b[2*i +: 2]);
            end
            result = tmp;
        end
        4'd5: begin
            tmp = 0;
            for (i = 0; i < QITS; i = i + 1) begin
                tmp[2*i +: 2] = a[2*i +: 2] ^ b[2*i +: 2];
            end
            result = tmp;
        end
        4'd6: result = (~a_val) & {WIDTH{1'b1}};
        default: result = a_val;
    endcase
end

endmodule

module q_alu_tb;
    parameter QITS = 8;
    reg  [2*QITS-1:0] a;
    reg  [2*QITS-1:0] b;
    reg  [3:0]        op;
    wire [2*QITS-1:0] result;

    q_alu #(.QITS(QITS)) dut (.a(a), .b(b), .op(op), .result(result));

    initial begin
        $display("Starting Q-ALU testbench");
        a = 16'h0003;
        b = 16'h0001;
        op = 4'd0;
        #10;
        if (result !== 16'h0004) $fatal("ADD mismatch");

        op = 4'd5;
        #10;
        if (result !== 16'h0002) $fatal("XOR mismatch");

        op = 4'd3;
        b = 16'h0002;
        #10;
        if (result !== 16'h0002) $fatal("MIN mismatch");

        op = 4'd4;
        #10;
        if (result !== 16'h0003) $fatal("MAX mismatch");

        $display("Q-ALU RTL testbench passed.");
        $finish;
    end
endmodule
