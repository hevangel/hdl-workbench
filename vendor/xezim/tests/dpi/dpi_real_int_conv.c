#include "svdpi.h"
#include <stdio.h>
extern int e_int(void);
extern long long e_long(void);
extern char e_byte(void);
extern short e_short(void);
extern svBit e_bit(void);
extern double e_real(void);
extern float e_sreal(void);
extern const char* e_str(void);
extern void e_out(int* o, long long* ol, double* orl);
extern unsigned int e_uint(void);
extern int e_half(void);
extern long long e_mhalf(void);
extern int e_rvar(void);
extern void sv_rep(const char* tag, long long v);
extern void sv_repr(const char* tag, double v);
extern void sv_reps(const char* tag, const char* v);
void c_entry(void) {
  int o; long long ol; double orl;
  sv_rep("exp int", e_int());
  sv_rep("exp long", e_long());
  sv_rep("exp byte", e_byte());
  sv_rep("exp short", e_short());
  sv_rep("exp bit", e_bit());
  sv_repr("exp real", e_real());
  sv_repr("exp sreal", e_sreal());
  sv_reps("exp str", e_str());
  e_out(&o, &ol, &orl);
  sv_rep("exp out o", o); sv_rep("exp out ol", ol); sv_repr("exp out orl", orl);
  sv_rep("exp uint", (long long)e_uint());
  sv_rep("exp half", e_half());
  sv_rep("exp mhalf", e_mhalf());
  sv_rep("exp rvar", e_rvar());
}
int c_add(int a, int b) { return a + b; }
double c_half(double x) { return x / 2; }
double c_ret_real(void) { return 2.5; }
long long c_long(long long x) { return x; }
void c_outs(int* o, double* r) { *o = 5; *r = 1.5; }
char c_byte(char b) { return b; }
