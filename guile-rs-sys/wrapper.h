#include <libguile.h>
#include <libguile/pairs.h>
#include <stdint.h>


void *rust_smob_data(SCM obj);

SCM rust_unspecified();

SCM rust_bool_true();

SCM rust_bool_false();

SCM scm_eol();

SCM scm_undefined();

SCM rust_car(SCM pair);

SCM rust_cdr(SCM pair);

SCM scm_make_char(uint32_t c);

SCM rust_new_smob(scm_t_bits tag, scm_t_bits data);

int scm_is_smob(scm_t_bits tag, SCM value);

int rust_is_heap_object(SCM value);

int scm_allow_other_keys = SCM_ALLOW_OTHER_KEYS;
int scm_allow_non_keyword_arguments = SCM_ALLOW_NON_KEYWORD_ARGUMENTS;