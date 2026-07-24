#ifndef __INCLUDE_H__
#define __INCLUDE_H__

#include "common.h"

#define TICKRATE    60

#define SAMPLERATE  22050

extern UINT videoPalette[];

void Audio_Callback(short *, int);
void Video_Viewport(int, int, int *, int *, int *, int *);

#endif
