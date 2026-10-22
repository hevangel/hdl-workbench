// top: c18_seed
class R; rand int v; endclass
module c18_seed;
  R a, b; int x1, x2, u1, u2, u3; string st; process p;
  initial begin
    a = new; b = new;
    a.srandom(42); b.srandom(42); void'(a.randomize()); void'(b.randomize());
    $display("T|18.13.3|srandom same seed equal=%0d", a.v == b.v);
    st = a.get_randstate(); void'(a.randomize()); x1 = a.v; a.set_randstate(st); void'(a.randomize()); x2 = a.v;
    $display("T|18.13.5|get/set randstate equal=%0d", x1 == x2);
    u1 = $urandom(7); u2 = $urandom(7); $display("T|18.13.1|urandom seed same=%0d", u1 == u2);
    u1 = $urandom_range(5, 3); $display("T|18.13.2|urandom_range rev ok=%0d", u1 >= 3 && u1 <= 5);
    u1 = $urandom_range(10); $display("T|18.13.2|urandom_range one ok=%0d", u1 <= 10);
    p = process::self(); st = p.get_randstate(); u1 = $urandom; p.set_randstate(st); u2 = $urandom;
    $display("T|18.13|process randstate equal=%0d", u1 == u2);
    p.srandom(5); u1 = $urandom; p.srandom(5); u2 = $urandom; $display("T|18.13|process srandom equal=%0d", u1 == u2);
    $display("T|18.14|value u1=%0d (thread-stable, compare across runs of same sim)", u1);
  end
  // random stability: adding unrelated thread must not change this thread's stream
  initial begin #1 $display("T|18.14|t2 first=%0d", $urandom); end
endmodule
