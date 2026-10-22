//! §7.2.1: grouped declarators retain declaration order in packed class fields.

#[test]
fn grouped_class_fields_match_flat_layout() {
    let sim = xezim::simulate(
        r#"
typedef struct packed { logic first, second, third; logic [4:0] tail; } bundle_t;
class holder;
  bundle_t payload;
  function new(); payload='0; endfunction
  function void fill(); payload.first=1; endfunction
endclass
module top;
  bundle_t direct;
  holder item;
  initial begin
    direct='0; direct.first=1;
    item=new(); item.fill();
    $display("PACKED %h %h",direct,item.payload);
    item.payload='0; item.payload.second=1;
    $display("MIDDLE %h",item.payload);
    item.payload='0; item.payload.third=1;
    $display("LOW %h",item.payload);
    item.payload=8'ha0;
    $display("READ %b%b%b",item.payload.first,item.payload.second,item.payload.third);
    $finish;
  end
endmodule
"#,
        100,
    )
    .expect("simulate");
    let output = sim
        .output
        .iter()
        .map(|line| line.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for expected in ["PACKED 80 80", "MIDDLE 40", "LOW 20", "READ 101"] {
        assert!(output.contains(expected), "missing {expected}: {output}");
    }
}
