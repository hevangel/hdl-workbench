//! §8.26.3: typedefs declared in an interface class are not inherited by a
//! class that implements it (sv-tests `type_access_implements_invalid`; the
//! reference simulator rejects `int_t` in the implementing class). The check
//! existed, but class-local typedefs are also registered in the design-wide
//! bare typedef table, so every interface-class typedef looked declared at
//! module scope and the check never fired.

use xezim::simulate;

const IFACE: &str = "
  interface class ihello;
    typedef int int_t;
    pure virtual function void hello(int_t val);
  endclass
";

#[test]
fn implementing_class_cannot_use_interface_typedef() {
    let src = format!(
        "module class_tb;{IFACE}
  class Hello implements ihello;
    virtual function void hello(int_t val); $display(\"hello %0d\", val); endfunction
  endclass
  Hello obj;
  initial begin obj = new; obj.hello(1); end
endmodule
"
    );
    assert!(simulate(&src, 10).is_err());
}

#[test]
fn class_scoped_reference_is_accepted() {
    let src = format!(
        "module class_tb;{IFACE}
  class Hello implements ihello;
    virtual function void hello(ihello::int_t val); $display(\"H|%0d\", val); endfunction
  endclass
  Hello obj;
  initial begin obj = new; obj.hello(3); end
endmodule
"
    );
    let sim = simulate(&src, 10).expect("simulate");
    assert!(sim.output.iter().any(|o| o.message == "H|3"));
}

#[test]
fn same_named_module_typedef_is_visible() {
    let src = format!(
        "module class_tb;
  typedef int int_t;{IFACE}
  class Hello implements ihello;
    virtual function void hello(int_t val); $display(\"M|%0d\", val); endfunction
  endclass
  Hello obj;
  initial begin obj = new; obj.hello(4); end
endmodule
"
    );
    let sim = simulate(&src, 10).expect("simulate");
    assert!(sim.output.iter().any(|o| o.message == "M|4"));
}
