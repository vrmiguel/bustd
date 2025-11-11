#include <sys/mman.h>

#ifdef MCL_ONFAULT
const int _MCL_ONFAULT = MCL_ONFAULT;
#else
const int _MCL_ONFAULT = -1;
#endif