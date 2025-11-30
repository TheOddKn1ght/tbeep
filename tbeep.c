#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <signal.h>
#include <string.h>
#include <ctype.h>
#include <stdbool.h>
#include <sys/wait.h>

#include "tbeep.h"

volatile sig_atomic_t running = 1;
volatile sig_atomic_t interrupted = 0;
static pid_t aplay_pid = 0;

void handle_sigint(int sig) {
    interrupted = 1;
    running = 0;

    if (aplay_pid > 0) {
        kill(aplay_pid, SIGTERM);
    }
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

int parse_duration(const char *str) {
    int total_seconds = 0;
    int value = 0;

    while (*str) {
        if (isdigit(*str)) {
            value = value * 10 + (*str - '0');
        } else {
            switch (*str) {
                case 'h': total_seconds += value * 3600; value = 0; break;
                case 'm': total_seconds += value * 60; value = 0; break;
                case 's': total_seconds += value; value = 0; break;
                default:
                    fprintf(stderr, "Invalid duration specifier: %c\n", *str);
                    exit(EXIT_FAILURE);
            }
        }
        str++;
    }

    if (value > 0) {
        total_seconds += value;
    }

    return total_seconds;
}

void print_time_left(int seconds_left) {
    int h = seconds_left / 3600;
    int m = (seconds_left % 3600) / 60;
    int s = seconds_left % 60;
    if (h > 0)
        printf("\rTime left: %02d:%02d:%02d   ", h, m, s);
    else
        printf("\rTime left: %02d:%02d   ", m, s);
    fflush(stdout);
}

void play_beep(const char *tmp_wav) {
    aplay_pid = fork();
    if (aplay_pid == 0) {
        freopen("/dev/null", "w", stderr);
        execlp("aplay", "aplay", "-q", tmp_wav, NULL);
        exit(EXIT_FAILURE);
    } else if (aplay_pid > 0) {
        int status;
        waitpid(aplay_pid, &status, 0);
        aplay_pid = 0;
    }
}

int main(int argc, char *argv[]) {
    bool verbose = false;
    const char *duration_arg = NULL;

    if (argc == 2) {
        if (strcmp(argv[1], "-h") == 0 || strcmp(argv[1], "--help") == 0) {
            printf("Usage: %s [-v] <duration>\n", argv[0]);
            printf(" (s)     -  seconds\n");
            printf("  m      -  minutes\n");
            printf("  h      -  hours \n");
            printf("\nOptions:\n");
            printf("  -v       Show countdown timer\n");
            printf("  -h       Show this help message\n");
            printf("\nPress Ctrl+C to stop the timer or beep.\n");
            return EXIT_SUCCESS;
        } else {
            duration_arg = argv[1];
        }
    } else if (argc == 3 && strcmp(argv[1], "-v") == 0) {
        verbose = true;
        duration_arg = argv[2];
    } else {
        fprintf(stderr, "Usage: %s [-v] <duration>\n", argv[0]);
        return EXIT_FAILURE;
    }

    int seconds = parse_duration(duration_arg);
    if (seconds <= 0) {
        fprintf(stderr, "Please enter a positive duration.\n");
        return EXIT_FAILURE;
    }

    signal(SIGINT, handle_sigint);

    char tmp_wav[] = "/tmp/beepXXXXXX.wav";
    int fd = mkstemps(tmp_wav, 4);
    if (fd == -1) {
        perror("mkstemps");
        return EXIT_FAILURE;
    }
    close(fd);
    write_temp_wav(tmp_wav);

    printf("Timer set for %d seconds...\n", seconds);

    if (verbose) {
        for (int i = seconds; i > 0 && running; --i) {
            print_time_left(i);
            sleep(1);
        }
        printf("\r%*s\r", 30, "");
    } else {
        for (int i = 0; i < seconds && running; ++i) {
            sleep(1);
        }
    }

    if (interrupted) {
        unlink(tmp_wav);
        printf("\nTimer cancelled.\n");
        return EXIT_SUCCESS;
    }

    printf("Time's up! Beeping... (Ctrl+C to stop)\n");

    while (running) {
        play_beep(tmp_wav);
        if (running) {
            usleep(500000);
        }
    }

    unlink(tmp_wav);
    printf("\nStopped.\n");
    return EXIT_SUCCESS;
}
