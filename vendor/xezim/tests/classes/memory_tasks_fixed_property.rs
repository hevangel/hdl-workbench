//! IEEE 1800-2017 §21.4: memory-loading tasks accept a fixed unpacked array
//! that is a class property, including a bare property name inside a method.

use xezim::simulate;

#[test]
fn readmemh_loads_a_fixed_class_property() {
    let dir = std::env::temp_dir().join(format!("xezim_class_mem_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let image = dir.join("words.hex");
    std::fs::write(&image, "a\n5\n").unwrap();
    let src = format!(
        r#"
module shell;
  class storage_box;
    logic [3:0] words [0:1];
    task fill(string path);
      $readmemh(path, words);
    endtask
  endclass
  initial begin
    storage_box item = new();
    item.fill("{}");
    $display("WORDS=%h,%h", item.words[0], item.words[1]);
  end
endmodule
"#,
        image.display()
    );
    let sim = simulate(&src, 100).expect("simulate");
    let text = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        text.contains("WORDS=a,5"),
        "fixed property was not loaded:\n{text}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
