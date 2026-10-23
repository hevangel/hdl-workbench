//! §25.3 / §7.6: interface-array slices bind physical instances in declared order.
use xezim::simulate;
fn output(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate")
        .output
        .iter()
        .map(|line| line.message.clone())
        .filter(|line| line.starts_with("T|"))
        .collect()
}
#[test]
fn slice_preserves_parameters_widths_and_input_connections() {
    let out = output(
        r#"interface bus_if #(parameter W=8)(input logic [W-1:0] src);
  wire [W-1:0] data=src;
  logic [W-1:0] ack;
endinterface
module sink #(parameter N=2)(bus_if b[0:N-1]);
  initial begin
    #1; $display("T|%m W=%0d,%0d bits=%0d data=%h,%h",b[0].W,b[1].W,$bits(b[0].data),b[0].data,b[1].data);
  end
endmodule
module top;
  logic [11:0] src=12'habc;
  bus_if #(.W(12)) bus[0:3](src);
  sink #(.N(4)) full(bus);
  sink #(.N(2)) sliced(bus[1:2]);
  initial begin #2; $finish; end
endmodule
"#,
    );
    assert!(
        out.contains(&"T|top.full W=12,12 bits=12 data=abc,abc".to_string()),
        "{out:?}"
    );
    assert!(
        out.contains(&"T|top.sliced W=12,12 bits=12 data=abc,abc".to_string()),
        "{out:?}"
    );
}

#[test]
fn slice_rebases_formal_bounds_and_preserves_reverse_writes() {
    let src = r#"
interface bus_if #(parameter W=12);
  logic [W-1:0] data, ack;
endinterface
module sink(bus_if b[7:6]);
  parameter I=7;
  initial begin
    #1; $display("T|%m data=%h,%h param=%h",b[7].data,b[6].data,b[I].data);
    b[7].ack=12'ha01; b[6].ack=12'ha02;
  end
endmodule
module top;
  bus_if a[0:3](), d[3:0]();
  for(genvar i=0;i<4;i++) begin : init_values
    initial begin a[i].data=12'h100+i; d[i].data=12'h200+i; end
  end
  sink up(a[1:2]);
  sink down(d[2:1]);
  initial begin #2;
    $display("T|ack=%h,%h,%h,%h",a[1].ack,a[2].ack,d[2].ack,d[1].ack); $finish;
  end
endmodule
"#;
    let out = output(src);
    for line in [
        "T|top.up data=101,102 param=101",
        "T|top.down data=202,201 param=202",
        "T|ack=a01,a02,a01,a02",
    ] {
        assert!(out.contains(&line.to_string()), "{out:?}");
    }
}

#[test]
fn forwarded_whole_view_and_subslice_keep_the_original_instances() {
    let src = r#"
interface bus_if #(parameter W=12); logic [W-1:0] data; endinterface
module leaf(bus_if b[0:1]);
  localparam WIDTH=b[0].W;
  logic [WIDTH-1:0] sized;
  initial begin #1; $display("T|leaf=%h,%h width=%0d,%0d",b[0].data,b[1].data,WIDTH,$bits(sized)); end
endmodule
module one(bus_if b[4:4]);
  initial begin #1; $display("T|one=%h W=%0d",b[4].data,b[4].W); end
endmodule
module mid(bus_if b[7:6]);
  leaf all(b);
  one part(b[6:6]);
endmodule
module top;
  bus_if #(.W(20)) bus[0:3]();
  for(genvar i=0;i<4;i++) begin : init_values
    initial bus[i].data=20'habc00+i;
  end
  mid u(bus[1:2]);
  initial begin #2; $finish; end
endmodule
"#;
    let out = output(src);
    for line in ["T|leaf=abc01,abc02 width=20,20", "T|one=abc02 W=20"] {
        assert!(out.contains(&line.to_string()), "{out:?}");
    }
}
