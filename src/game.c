#include "misc.h"
#include "audio.h"
#include "video.h"

#include "game.h"

// Ported to Rust
extern void DoPauseDrawer();
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

void ClockTicker()
{
    // 256 frames = 1 game minute
    // 19 game hours = 6.75... actual hours (19 * 60 * 256 / 12 / 60 / 60)
    // there's a guy on YouTube that can do it in less than 20m
    //  (2m15s using cheat mode)
    if (gameClockTicks++ < 256)
    {
        return;
    }

    gameClockTicks = 0;

    gameScoreClock[0]++;
    if (gameScoreClock[0] == 60)
    {
        gameScoreClock[0] = 0;
        gameScoreClock[1]++;
        if (gameScoreClock[1] == 12)
        {
            gameScoreClock[2] = 1 - gameScoreClock[2];
            if (gameScoreClock[2] == 0 && gameMode < GM_MARIA)
            {
                Action = Gameover_Action;
            }
        }
        else if (gameScoreClock[1] == 13)
        {
            gameScoreClock[1] = 1;
        }
    }

    DoClockUpdate = DoDrawClock;
}


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

void DoGameTicker()
{
    if (gameMusic == MUS_STOP && gameInactivityTimer++ == 256 * 5 && gameMode < GM_RUNNING)
    {
        Game_Pause(1);

        return;
    }

    if (gameMusic == MUS_PLAY)
    {
        Miner_IncSeq();
    }

    gameFrame = Timer_Update(&gameTimer);
    if (gameFrame == 0)
    {
        return;
    }

    Level_Ticker();
    Robots_Ticker();

    if (gameMode == GM_TOILET)
    {
        if (gameClockTicks++ == 256)
        {
            Action = Title_Action;
        }

        return;
    }

    Miner_Ticker();

    if (gameMode == GM_RUNNING)
    {
        minerWilly.frame |= 1;

        if (minerWilly.x == 224 && gameLevel == THEBATHROOM)
        {
            gameMode = GM_TOILET;
            Robots_Flush();
            gameClockTicks = 0;
        }

        return;
    }

    if (gameMode == GM_MARIA && gameLevel == MASTERBEDROOM)
    {
        if (minerWilly.air == 0 && minerWilly.x == 40)
        {
            gameMode = GM_RUNNING;
        }
    }

    Rope_Ticker();

    ClockTicker();
}

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
