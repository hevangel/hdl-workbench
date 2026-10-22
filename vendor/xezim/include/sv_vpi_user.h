#ifndef SV_VPI_USER_H
#define SV_VPI_USER_H

/* Compatibility shim for the Accellera UVM reference implementation.
 *
 * Some UVM source files (`uvm_hdl_polling.c` and a vendor-specific backend)
 * `#include "sv_vpi_user.h"` directly. The full Accellera header
 * declares vpiTypes.h contents and the vlog_chk_error / io_printf
 * family of legacy PLI v1.0 helpers — xezim doesn't implement
 * those, but the C compile needs the include chain to resolve.
 *
 * The actual types UVM uses (`svScope`, `svLogicVecVal`) now live
 * in `svdpi.h` per IEEE 1800 §35.5.5. This file just re-includes
 * `svdpi.h` and provides a few extra typedefs the legacy header
 * is expected to define.
 */

#include "svdpi.h"
#include "vpi_user.h"

/* Legacy typedefs from the full sv_vpi_user.h that some UVM source
 * files still expect to be visible after `#include "sv_vpi_user.h"`.
 * The full Accellera header defines these via vpi_user.h + vpi_compatibility.h;
 * for xezim, both types are already available from vpi_user.h. */


/* Generic instance class (module/program/interface/package). Accepted by
 * vpi_iterate to enumerate the design root and a scope's child instances. */
#define vpiInstance 745

/* --- Object model (IEEE 1800-2017 chapter 37): SystemVerilog objects ---- */

/* Object types. */
#define vpiPackage           600
#define vpiInterface         601
#define vpiProgram           602
#define vpiTypespec          605   /* also the vpi_handle relation */
#define vpiModport           606   /* an interface's modport; vpiIODecl iterates its ports */
#define vpiRefObj            608   /* a name the object model cannot resolve */
#define vpiVarBit            vpiRegBit
#define vpiArrayVar          vpiRegArray
#define vpiClassVar          615
#define vpiChandleVar        622
#define vpiPackedArrayVar    623
#define vpiVirtualInterfaceVar 728
#define vpiLongIntTypespec   625
#define vpiShortRealTypespec 626
#define vpiByteTypespec      627
#define vpiShortIntTypespec  628
#define vpiIntTypespec       629
#define vpiClassTypespec     630
#define vpiStringTypespec    631
#define vpiChandleTypespec   632
#define vpiEnumTypespec      633
#define vpiEnumConst         634   /* also the vpi_iterate relation */
#define vpiIntegerTypespec   635
#define vpiTimeTypespec      636
#define vpiRealTypespec      637
#define vpiStructTypespec    638
#define vpiUnionTypespec     639
#define vpiBitTypespec       640
#define vpiLogicTypespec     641
#define vpiArrayTypespec     642
#define vpiVoidTypespec      643
#define vpiTypespecMember    644   /* also the vpi_iterate relation */
#define vpiPackedArrayTypespec 692
#define vpiEventTypespec     698
#define vpiFinal             676
#define vpiArrayNet          vpiNetArray
#define vpiEnumNet           680
#define vpiIntegerNet        681
#define vpiLogicNet          vpiNet
#define vpiTimeNet           682
#define vpiStructNet         683
#define vpiPackedArrayNet    693

/* Relations. */
#define vpiBaseTypespec      703
#define vpiElemTypespec      704
#define vpiMember            742

/* Properties and their values. */
#define vpiTop               600
#define vpiUnit              602
#define vpiJoinType          603
#define vpiJoin                0
#define vpiJoinNone            1
#define vpiJoinAny             2
#define vpiAccessType        604
#define vpiForkJoinAcc         1
#define vpiExternAcc           2
#define vpiDPIExportAcc        3
#define vpiDPIImportAcc        4
#define vpiArrayType         606
#define vpiStaticArray         1
#define vpiDynamicArray        2
#define vpiAssocArray          3
#define vpiQueueArray          4
#define vpiArrayMember       607
#define vpiPortType          611
#define vpiInterfacePort       1
#define vpiModportPort         2
#define vpiConstantVariable  612
#define vpiStructUnionMember 615
#define vpiVisibility        620
#define vpiPublicVis           1
#define vpiProtectedVis        2
#define vpiLocalVis            3
#define vpiAlwaysType        624
#define vpiAlwaysComb          2
#define vpiAlwaysFF            3
#define vpiAlwaysLatch         4
#define vpiPacked            630
#define vpiTagged            632
#define vpiRef                 6   /* vpiDirection of a ref port or argument */
#define vpiDPIPure           665
#define vpiDPIContext        666
#define vpiOtherFunc           6   /* vpiFuncType: returns a non-integral type */

/* vpiOpType values added by SystemVerilog. */
#define vpiPostIncOp          62
#define vpiPreIncOp           63
#define vpiPostDecOp          64
#define vpiPreDecOp           65
#define vpiWildEqOp           69
#define vpiWildNeqOp          70
#define vpiStreamLROp         71
#define vpiStreamRLOp         72
#define vpiAssignmentPatternOp 75
#define vpiInsideOp           95

#endif /* SV_VPI_USER_H */