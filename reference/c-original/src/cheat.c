#include "common.h"
#include "game.h"

char        cheatCode[] = "writetyper.";
int         cheatEnabled = 0;
int         cheatPos = 0;
int         cheatLevel[2];
EVENT       Cheat_Responder = Cheat_Disabled;

void Cheat_Enabled()
{
    int         level;

    if (gameInput == KEY_ENTER && cheatPos > 0)
    {
        level = cheatLevel[0];
        if (cheatPos == 2)
        {
            level = level * 10 + cheatLevel[1];
        }
        level--;

        cheatPos = 0;

        if (level < 0 || level > 59 || level == gameLevel)
        {
            return;
        }

        gameLevel = level;
        Ticker = Game_InitRoom; // NOT Action
        return;
    }

    if (gameInput < KEY_0 || gameInput > KEY_9)
    {
        cheatPos = 0;
        return;
    }

    if (cheatPos == 2)
    {
        cheatPos = 0;
        return;
    }

    cheatLevel[cheatPos] = gameInput - KEY_0;
    cheatPos++;
}

void Cheat_Disabled()
{
    if (gameLevel != FIRSTLANDING || minerWilly.y != 104)
    {
        cheatPos = 0;
        return;
    }

    if (gameInput < KEY_A || gameInput > KEY_Z)
    {
        return;
    }

    if (cheatCode[cheatPos] != gameInput - KEY_A + 'a')
    {
        cheatPos = 0;
        return;
    }

    cheatPos++;
    if (cheatCode[cheatPos] != '.')
    {
        return;
    }

    cheatEnabled = 1;
    cheatPos = 0;

    Cheat_Responder = Cheat_Enabled;
    Robots_DrawCheat();
}
