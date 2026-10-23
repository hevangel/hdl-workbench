use std::path::Path;
use std::process::Command;

/// Pure-SystemVerilog self-test: a `foreach` over a rand FIXED ARRAY OF
/// PACKED STRUCTS whose body constrains a MEMBER of each element — the 5446
/// reg-map-config shape:
///
///     rand uvm_reg_map_cfg_t cfg[2][5];
///     constraint legal {
///       foreach (cfg[x,y]) {
///         cfg[x][y].au_bytes inside {1,2,4,8,16};
///         cfg[x][y].endian  inside {UVM_LITTLE_ENDIAN};
///       }
///     }
///
/// The fixed-array foreach repair only matched a BARE element target
/// (`cfg[y]`); `index_chain_root_is` is blind to the `MemberAccess` wrapping
/// the `Index`, so `cfg[y].au_bytes inside` died in the generate-and-test
/// fallback (whole 65-bit element drawn uniformly → never a 32-bit member in
/// {1,2,4,8,16}) and randomize() returned 0 (the 5446 failure). Now the
/// member slice is re-picked within its range and spliced back into the
/// element.
#[test]
fn foreach_packed_struct_member_inside() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let test_file = test_dir.join("classes/foreach_packed_struct_member_inside.sv");
    assert!(
        test_file.exists(),
        "Test file not found: {}",
        test_file.display()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .arg(test_file.to_str().unwrap())
        .output()
        .expect("Failed to execute xezim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    assert!(
        !combined.contains("Parse errors"),
        "Parse error in foreach_packed_struct_member_inside.sv:\n{combined}"
    );
    assert!(
        !combined.contains("Simulation error"),
        "Simulation error in foreach_packed_struct_member_inside.sv:\n{combined}"
    );
    assert!(
        combined.contains("TAG_PASS"),
        "Rand fixed-array-of-packed-struct member `inside` was not solved.\nOutput:\n{combined}"
    );
}
