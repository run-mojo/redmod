//
// Created by Clay Molocznik on 7/11/18.
//

#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <stdio.h>
#include <errno.h>
#include <math.h>
#include "rax_ext.h"

#ifndef RAX_MALLOC_INCLUDE
#define RAX_MALLOC_INCLUDE "rax_malloc.h"
#endif

#include RAX_MALLOC_INCLUDE "rax_malloc.h"

raxIterator *raxIteratorNew(rax *rt) {
    raxIterator *it = rax_malloc(sizeof(raxIterator));
    raxStart(it, rt);
    return it;
}

void raxIteratorFree(raxIterator *it) {
    rax_free(it);
}

#include "rax_ext.h"
