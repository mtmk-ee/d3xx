

#if defined (__linux__) || defined (__APPLE__)
#include "linux/ftd3xx.h"
#elif defined (_WIN32)
#include "windows/ftd3xx.h"
#endif
