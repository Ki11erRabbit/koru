#include <libguile.h>
#include <libguile/pairs.h>
#include <stdint.h>


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

SCM scm_eol() {
    return SCM_EOL;
}

SCM scm_undefined() {
    return SCM_UNDEFINED;
}

SCM rust_car(SCM pair) {
    return SCM_CAR(pair);
}

SCM rust_cdr(SCM pair) {
    return SCM_CDR(pair);
}

SCM scm_make_char(uint32_t c) {
    return SCM_MAKE_CHAR(c);
}

SCM rust_new_smob(scm_t_bits tag, scm_t_bits data) {
    return scm_new_smob(tag, data);
}