//
// Created by Clay Molocznik on 7/13/18.
//

#ifndef RAX_STREAM_EXT_H
#define RAX_STREAM_EXT_H

#include "stream.h"
//#include "server.h"

int64_t lpGetInteger(unsigned char *ele);
unsigned char *lpAppendInteger(unsigned char *lp, int64_t value);
unsigned char *lpReplaceInteger(unsigned char *lp, unsigned char **pos, int64_t value);

void streamEncodeID(void *buf, streamID *id);
void streamDecodeID(void *buf, streamID *id);
int streamCompareID(streamID *a, streamID *b);

int streamAppendItemSDSMap(stream *s, void **argv, int64_t numfields, streamID *added_id, streamID *use_id);
void streamNextID(streamID *last_id, streamID *new_id);

#endif //RAX_STREAM_EXT_H
