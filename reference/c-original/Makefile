# jetset willy

YEAR = $(shell date +'%Y')

TARGET = jetsetwilly

CC = gcc

CFLAGS = -g -Wall -Wextra -MMD
LDFLAGS = -lSDL2

DEFINES = -DBUILD=\"v1.0.$(YEAR)\"
DEFINES += -DNOCODES

SRC = src
O = linux

OBJS = $(O)/main.o $(O)/system.o $(O)/video.o $(O)/loader.o $(O)/title.o $(O)/audio.o $(O)/miner.o $(O)/levels.o $(O)/game.o $(O)/die.o $(O)/gameover.o $(O)/robots.o $(O)/rope.o $(O)/cheat.o $(O)/misc.o $(O)/codes.o

all:	dir $(OBJS)
	$(CC) $(OBJS) -o $(TARGET) $(LDFLAGS)

$(O)/loader.o:	$(SRC)/loader.c
	$(CC) $(CFLAGS) $(DEFINES) -c $< -o $@

$(O)/%.o:	$(SRC)/%.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -rf $(O) $(TARGET)

install: all
	cp $(TARGET) ~/.local/bin/

uninstall:
	rm ~/.local/bin/$(TARGET)

dir:
	@mkdir -p $(O)

-include $(O)/*.d

# jetset willy
