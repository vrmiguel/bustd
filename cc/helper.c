#include <sys/mman.h>

int _mcl_onfault() {
    return MCL_ONFAULT;
}

int _char_is_signed() {
    return (char)-1 < 0;
}