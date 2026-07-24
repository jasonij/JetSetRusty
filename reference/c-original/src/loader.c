#include "misc.h"
#include "video.h"
#include "audio.h"
#include "system.h"

#ifndef NOCODES
#define NEXT_ACTION Codes_Action
#else
#define NEXT_ACTION Title_Action
#endif

int     loaderTicks = 0;
char    loaderText[25] = "\x1\x7\x2\x2" "JetSet Willy Loading";
int     loaderFlash = 0;

void DoLoaderResponder()
{
    Action = NEXT_ACTION;
}

void DoLoaderTicker()
{
    BYTE    swap;

    if (loaderFlash != videoFlash)
    {
        swap = loaderText[3];
        loaderText[3] = loaderText[1];
        loaderText[1] = swap;
        loaderFlash = videoFlash;
    }

    if (loaderTicks++ == 256)
    {
        Action = NEXT_ACTION;
    }
}

void DoLoaderDrawer3()
{
    Video_WriteLarge(6 * 8, 11 * 8, loaderText);
}

void DoLoaderDrawer2()
{
    if (audioMusicPlaying)
    {
        return;
    }

    Video_Write(80 * WIDTH + 5 * 8, "\x1\x6\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14");
    Video_Write(88 * WIDTH + 5 * 8, "\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14");
    Video_Write(96 * WIDTH + 5 * 8, "\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14");
    Video_Write(104 * WIDTH + 5 * 8, "\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14\x14");
    DoLoaderDrawer3();

    Responder = DoLoaderResponder;
    Ticker = DoLoaderTicker;
    Drawer = DoLoaderDrawer3;
}

void DoLoaderDrawer1()
{
    System_Border(0x1);
    Video_PixelPaperFill(0, WIDTH * HEIGHT, 0x1);
    Video_Write(23 * 8 * WIDTH, "\x1\x1\x2\x7" "fawtytoo");
    Video_Write(23 * 8 * WIDTH + WIDTH - Video_TextWidth(BUILD), "\x2\x0" BUILD);

    Audio_Music(MUS_LOADER, MUS_PLAY);

    Drawer = DoLoaderDrawer2;
}

void Loader_Action()
{
    Drawer = DoLoaderDrawer1;

    Action = DoNothing;
}
