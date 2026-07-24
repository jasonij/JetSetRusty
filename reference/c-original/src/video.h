#define B_LEVEL   1
#define B_ROBOT   2
#define B_WILLY   4

#define SPRITETILE(t)   (((t & 6) << 4) | (t & 1))
#define TILE2PIXEL(t)   (((t & 992) << 6) | ((t & 31) << 3))
#define YALIGN(y)       (4 | ((y & 4) >> 1) | (y & 2) | ((y & 1) << 1))

void Video_Draw(void);
void Video_Write(int, char *);
void Video_WriteLarge(int, int, char *);
void Video_DrawSprite(int, WORD *, BYTE, BYTE);
void Video_DrawRobot(int, WORD *, BYTE);
void Video_DrawMiner(int, WORD *, int);
void Video_DrawTile(int, BYTE *, BYTE, BYTE);
void Video_DrawArrow(int, int);
void Video_DrawRopeSeg(int, BYTE);
void Video_PixelFill(int, int);
void Video_PixelInkFill(int, int, BYTE);
void Video_PixelPaperFill(int, int, BYTE);
void Video_CycleColours(int);
int Video_TextWidth(char *);
int Video_GetPixel(int);
