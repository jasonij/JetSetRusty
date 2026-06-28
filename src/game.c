#include "misc.h"
#include "audio.h"
#include "video.h"

#include "game.h"

// Ported to Rust
extern void DoPauseDrawer();
extern void DoGameTicker();
extern void Game_GameReset();
extern void Game_CheatEnabled();
extern void Game_ChangeLevel(int dir);

int             gameMusic = MUS_PLAY;

int             levelBorder[] =
{
    5, 4, 6, 2, 3, 1, 2, 1, 4, 2,
    2, 4, 6, 5, 1, 3, 2, 1, 2, 1,
    2, 1, 4, 4, 1, 1, 5, 2, 3, 2,
    2, 2, 2, 2, 1, 1, 5, 6, 2, 2,
    1, 1, 2, 5, 3, 4, 1, 2, 4, 5,
    5, 2, 1, 2, 5, 1, 2, 2, 5, 5
};

char            gameScoreItems;
char            gameScoreClock[3];
static EVENT    DoClockUpdate;

int             gameInactivityTimer;

int      gameFrame;
TIMER           gameTimer;

int             gamePaused = 0;
int             gameLevel;
int             gameLives;
int             gameClockTicks;
int             gameMode;

int             itemCount;

// Game_ChangeLevel - ported to game.rs
// ClockTicker - ported to game.rs as clock_ticker


// DoPauseDrawer - ported to game.rs

void DoGameDrawer()
{
    if (gameMusic == MUS_PLAY)
    {
        GameDrawLives();
    }

    if (gameFrame == 0)
    {
        return;
    }

    Level_Drawer();
    Robots_Drawer();

    if (gameMode == GM_TOILET)
    {
        return;
    }

    Miner_Drawer();
    Rope_Drawer();

    DoClockUpdate();
}

// DoGameTicker - ported to game.rs

void Game_Pause(int state)
{
    if (gamePaused == state || gameMode >= GM_RUNNING)
    {
        return;
    }

    gamePaused = state;

    if (gamePaused)
    {
        if (cheatEnabled)
        {
            Ticker = DoNothing;
            Drawer = DoNothing;
        }
        else
        {
            Ticker = DoPauseTicker;
            Drawer = DoPauseDrawer;
        }
        Audio_Play(MUS_STOP);
    }
    else
    {
        Ticker = DoGameTicker;
        Drawer = DoGameDrawer;
        Audio_Play(gameMusic);

        gameInactivityTimer = 0;
        if (cheatEnabled == 0)
        {
            Game_DrawStatus();
            System_Border(levelBorder[gameLevel]);
        }
    }
}

// Game_CheatEnabled - ported to game.rs

static void DoGameResponder()
{
    gameInactivityTimer = 0;

    if (gameInput == KEY_PAUSE)
    {
        Game_Pause(gamePaused ? 0 : 1);
    }
    else if (gameInput == KEY_MUTE)
    {
        gameMusic = gameMusic == MUS_PLAY ? MUS_STOP : MUS_PLAY;
        Audio_Play(gameMusic);

        Game_Pause(0);
    }
    else if (gameInput == KEY_ESCAPE)
    {
        Action = Title_Action;
    }
    else
    {
        Cheat_Responder();
    }
}

void Game_InitRoom()
{
    Level_Init();
    Robots_Init();
    Rope_Init();
    System_Border(levelBorder[gameLevel]);
    Miner_Save();

    minerAttrSplit = 6;
    if (gameLevel == SWIMMINGPOOL)
    {
        minerAttrSplit = 5; // willy goes blue when underwater
    }

    Timer_Set(&gameTimer, 12, TICKRATE);
    gameFrame = 1;
    gameInactivityTimer = 0;

    minerWillyRope = 0;

    if (gamePaused)
    {
        Ticker = DoNothing;
        Drawer = DoDrawOnce;
    }
    else
    {
        Ticker = DoGameTicker;
    }

    Action = DoNothing;
}

// Game_GameReset - ported to game.rs

void Game_Action_ORIG()
{
    Responder = DoGameResponder;
    Ticker = Game_InitRoom;
    Drawer = DoGameDrawer;
}
