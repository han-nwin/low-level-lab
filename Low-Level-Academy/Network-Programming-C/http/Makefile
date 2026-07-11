CC = gcc
CFLAGS = -Wall -Wextra -pedantic -std=c11

all: http server-select server-poll

http: http.c
	$(CC) $(CFLAGS) -o $@ $<

server-select: server-select.c
	$(CC) $(CFLAGS) -o $@ $<

server-poll: server-poll.c
	$(CC) $(CFLAGS) -o $@ $<

clean:
	rm -f http server-select server-poll
