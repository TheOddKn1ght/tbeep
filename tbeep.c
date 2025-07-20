#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <signal.h>
#include <string.h>

#include "tbeep.h"

volatile sig_atomic_t running = 1;

void handle_sigint(int sig) {
    running = 0;
}

void write_temp_wav(const char *filename) {
    FILE *f = fopen(filename, "wb");
    if (!f) {
        perror("fopen");
        exit(EXIT_FAILURE);
    }
    fwrite(beep_wav, sizeof(beep_wav), 1, f);
    fclose(f);
}

int main(int argc, char *argv[]) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s <seconds>\n", argv[0]);
        return EXIT_FAILURE;
    }

    int seconds = atoi(argv[1]);
    if (seconds <= 0) {
        fprintf(stderr, "Please enter a positive number of seconds.\n");
        return EXIT_FAILURE;
    }

    char tmp_wav[] = "/tmp/beepXXXXXX.wav";
    int fd = mkstemps(tmp_wav, 4);
    if (fd == -1) {
        perror("mkstemps");
        return EXIT_FAILURE;
    }
    close(fd);
    write_temp_wav(tmp_wav);

    printf("Timer set for %d seconds...\n", seconds);
    sleep(seconds);

    signal(SIGINT, handle_sigint);
    printf("Time's up! Beeping... (Ctrl+C to stop)\n");

    char cmd[512];
    snprintf(cmd, sizeof(cmd), "aplay -q \"%s\"", tmp_wav);

    while (running) {
        system(cmd);
        usleep(500000);
    }

    unlink(tmp_wav); // Delete temp file
    printf("\nStopped.\n");
    return EXIT_SUCCESS;
}
