#include "common.h"
#include "video.h"
#include "audio.h"
#include "system.h"
#include "game.h"

WORD    plinthSprite[16] = {14316, 30702, 0, 28662, 61431, 61431, 54619, 56251, 54619, 57339, 60791, 61175, 28022, 0, 30702, 14316};
WORD    bootSprite[16] = {4224, 4224, 4224, 4224, 4224, 4224, 4224, 8320, 8320, 18498, 34869, 33801, 32769, 32770, 17293, 15478};

int     bootTicks;

char    textGame[18] = "\x1\x0\x2\x0" "G " "\x2\x0" "a " "\x2\x0" "m " "\x2\x0" "e";
char    textOver[18] = "\x1\x0\x2\x0" "O " "\x2\x0" "v " "\x2\x0" "e " "\x2\x0" "r";

void Gameover_Drawer()
{
    if (bootTicks <= 96)
    {
        Video_DrawSprite((bootTicks & 126) * WIDTH + 15 * 8, bootSprite, 0x0, 0x7);
        Video_PixelPaperFill(0, 128 * WIDTH, (bootTicks & 12) >> 2);
    }

    if (bootTicks < 96)
    {
        return;
    }

    Video_WriteLarge(7 * 8, 6 * 8, textGame);
    Video_WriteLarge(18 * 8, 6 * 8, textOver);
}

void Gameover_Ticker()
{
    int c = bootTicks >> 2;

    textGame[3] = c++ & 0x7;
    textGame[7] = c++ & 0x7;
    textGame[11] = c++ & 0x7;
    textGame[15] = c++ & 0x7;
    textOver[3] = c++ & 0x7;
    textOver[7] = c++ & 0x7;
    textOver[11] = c++ & 0x7;
    textOver[15] = c++ & 0x7;

    bootTicks++;

    if (bootTicks < 256)
    {
        return;
    }

    Action = Title_Action;
}

void Gameover_Init()
{
    System_Border(0x0);
    Video_PixelFill(0, 128 * WIDTH);
    Video_DrawSprite(96 * WIDTH + 15 * 8, minerSprite[4 + 2], 0x0, 0x7);
    Video_DrawSprite(112 * WIDTH + 15 * 8, plinthSprite, 0x0, 0x2);
    bootTicks = 0;

    Audio_Play(MUS_STOP);
    Audio_Sfx(SFX_GAMEOVER);

    Ticker = Gameover_Ticker;
}

void Gameover_Action()
{
    Responder = DoNothing;
    Ticker = Gameover_Init;
    Drawer = Gameover_Drawer;

    Action = DoNothing;
}
