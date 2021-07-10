#include <sys/mman.h>

int _mcl_onfault() {
    return MCL_ONFAULT;
}