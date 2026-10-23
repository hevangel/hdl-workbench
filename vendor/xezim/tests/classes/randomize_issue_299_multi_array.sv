module mwe16_multi_array_foreach_obf ;
   typedef logic [6:0] w7_t ;
   typedef logic [8:0] w9_t ;
   typedef enum logic [1:0] { MODE_TURN_OFF, MODE_RESET_AND_TURN_ON,
                              MODE_RESET_ONLY, MODE_TURN_ON } mode_ctrl_op_t ;
   typedef enum logic [1:0] { RD_ALL_ROWS_OF_RANDOM_GRP,
                              RD_SINGLE_ROW_OF_ALL_GRPS } mode_pick_t ;
   int fail_cnt = 0 ;
   task automatic check(bit ok, string msg);
      if (!ok) begin fail_cnt++ ; $display("FAIL : %s", msg) ; end
   endtask

   // =====================================================================
   // FAILING t1 (from mwe10, probeC C6 true minimum): TWO arrays, foreach
   // elem == index, in a task
   // =====================================================================
   task automatic t1_two_arrays();
      w7_t p [] ; w7_t q [] ;
      if (std::randomize(p, q) with {
            p.size() == 37 ; q.size() == 37 ;
            foreach (p[k]) { p[k] == k ; q[k] == k ; }
         }) begin
         $display("OK t1 two-array: size=%0d p[3]=%0d q[3]=%0d", p.size(), p[3], q[3]) ;
         check(p.size() == 37 && p[3] == 3 && q[3] == 3, "t1: elem != idx") ;
      end else check(0, "t1 two-array: std::randomize returned 0 (spurious)") ;
   endtask

   // =====================================================================
   // CONTROL c1 (from mwe10): ONE array, same foreach: passes under xezim
   // =====================================================================
   task automatic c1_one_array();
      w7_t p [] ;
      if (std::randomize(p) with {
            p.size() == 37 ;
            foreach (p[k]) p[k] == k ;
         }) check(p.size() == 37 && p[3] == 3, "c1: elem != idx") ;
      else check(0, "c1 one-array: std::randomize returned 0") ;
   endtask

   initial begin
      t1_two_arrays() ;            // <-- fails under xezim

      // ---- FAILING t2 (from mwe8, site S21): full shape, INLINE ----
      // Four dynamic arrays + enum in one solve set; the branch condition
      // (mode_pick) is NOT in the solve set, so the solver must take
      // the then-branch with constant sizes and pin every element to its
      // index. Fails inline as well as in a task (intrinsic solver bug).
      begin
         w7_t src_pick_list [], pace_pick_list [], gate_pick_list [] ;
         w9_t slot_idx_list [] ;
         mode_ctrl_op_t mode_op ;
         mode_pick_t mode_pick ;
         mode_pick = RD_ALL_ROWS_OF_RANDOM_GRP ;
         if (std::randomize(src_pick_list, pace_pick_list, gate_pick_list, slot_idx_list, mode_op) with {
            if (mode_pick == RD_ALL_ROWS_OF_RANDOM_GRP) {
               src_pick_list.size()   == 37 ;
               pace_pick_list.size()  == 37 ;
               gate_pick_list.size()  == 37 ;
               slot_idx_list.size()   == 272 ;
               foreach (src_pick_list[k]) {
                  src_pick_list[k]   == k ;
                  pace_pick_list[k]  == k ;
                  gate_pick_list[k]  == k ;
               }
               foreach (slot_idx_list[k]) {
                  slot_idx_list[k]   == k ;
               }
            } else {
               src_pick_list.size()   == 1 ;
               pace_pick_list.size()  == 1 ;
               gate_pick_list.size()  == 1 ;
               slot_idx_list.size()   == 1 ;
               foreach (src_pick_list[k]) {
                  src_pick_list[k]   inside { [0:36] };
                  pace_pick_list[k]  inside { [0:36] };
                  gate_pick_list[k]  inside { [0:36] };
               }
            }
            mode_op dist {
               MODE_TURN_OFF := 1,
               MODE_RESET_AND_TURN_ON := 12,
               MODE_RESET_ONLY := 1,
               MODE_TURN_ON := 7
            };
         }) begin
            $display("OK t2 sizes %0d %0d op=%0d", src_pick_list.size(), slot_idx_list.size(), mode_op) ;
            foreach (src_pick_list[k]) check(src_pick_list[k] == k, "t2: src_pick_list[k] != k") ;
         end else check(0, "t2 full S21 shape: std::randomize returned 0 (spurious)") ;
      end

      c1_one_array() ;             // control: passes under xezim
      if (fail_cnt == 0) $display("TEST_PASS");
      else begin $display("TEST_FAIL count=%0d", fail_cnt); $fatal(1); end
   end
endmodule
