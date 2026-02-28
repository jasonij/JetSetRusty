#define MUS_STOP    0
#define MUS_PLAY    1

enum
{
    MUS_TITLE,
    MUS_GAME,
    MUS_LOADER
};

enum
{
    SFX_ITEM,
    SFX_DIE,
    SFX_GAMEOVER,
    SFX_ARROW,
    SFX_WILLY,
    SFX_NONE
};

extern int  audioMusicPlaying;
extern int  audioPanX;

void Audio_Init(void);
void Audio_Music(int, int);
void Audio_Play(int);
void Audio_WillySfx(int, int);
void Audio_Sfx(int);
void Audio_ReduceMusicSpeed(void);
