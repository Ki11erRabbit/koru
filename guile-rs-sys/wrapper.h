#include <libguile.h>


void *rust_smob_data(SCM obj) {
    return (void*)SCM_SMOB_DATA(obj);
}

SCM rust_unspecified() {
    return SCM_UNSPECIFIED;
}

SCM rust_bool_true() {
    return SCM_BOOL_T;
}

SCM rust_bool_false() {
    return SCM_BOOL_F;
}