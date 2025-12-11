#ifndef _FAKE_STDINT_H
#define _FAKE_STDINT_H

typedef signed char         int8_t;
typedef unsigned char       uint8_t;

typedef short               int16_t;
typedef unsigned short      uint16_t;

typedef int                 int32_t;
typedef unsigned int        uint32_t;

typedef long long           int64_t;
typedef unsigned long long  uint64_t;

typedef long long           intptr_t;
typedef unsigned long long  uintptr_t;

#define NULL ((void*)0)

typedef unsigned char bool;
#define true 1
#define false 0

typedef uint64_t phys_addr_t;

#endif
