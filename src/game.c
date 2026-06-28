#include "misc.h"
#include "audio.h"
#include "video.h"

#include "game.h"

// All functions below have been ported to game.rs
// The Rust versions are marked pub extern "C" and visible to C automatically

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
// DoGameTicker - ported to game.rs
// DoGameDrawer - ported to game.rs as do_game_drawer
// Game_Pause - ported to game.rs as game_pause
// Game_CheatEnabled - ported to game.rs
// Game_ChangeLevel - ported to game.rs
// DoGameResponder - ported to game.rs as do_game_responder
// Game_InitRoom - ported to game.rs as game_init_room
// ClockTicker - ported to game.rs as clock_ticker
// Game_Action_ORIG - reference copy, not needed
