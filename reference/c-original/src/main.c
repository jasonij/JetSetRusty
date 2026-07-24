#include "common.h"
#include "system.h"
#include "video.h"
#include "audio.h"

EVENT   Action = Loader_Action;
EVENT   Responder = DoNothing;
EVENT   Ticker = DoNothing;
EVENT   Drawer = DoNothing;

int     gameRunning = 1, gameInput;

int     videoFlash = 0;

void DoNothing()
{
}

void DoQuit()
{
    gameRunning = 0;
    Drawer = DoNothing;
    Ticker = DoNothing;
}

int main()
{
    int flash = 0;

    Audio_Init();
    System_Init();

    while (gameRunning)
    {
        System_StartFrame();

        Action();

        while (System_PollKeys(&gameInput))
        {
            Responder();
        }

        Ticker();
        Drawer();

        Video_Draw();
        System_VideoUpdate();

        flash++;
        if (flash == 20)
        {
            flash = 0;
            videoFlash = 1 - videoFlash;
        }
    }

    System_Quit();

    return 0;
}
