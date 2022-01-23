// `bustd`'s memory eater

#include <stdio.h>
#include <time.h>
#include <unistd.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

struct free_mem_s {
    unsigned available_mem_mib;
    unsigned available_swap_mib;
};

typedef struct free_mem_s free_mem_t;

void display(free_mem_t * mem, float psi) {
    printf("\rFree RAM: %d MiB. Free swap: %d MiB. PSI: %0.2f", mem->available_mem_mib, mem->available_swap_mib, psi);
    fflush(stdout);
}

float memory_pressure_some_avg_10(void) {
    FILE * memory_pressure = fopen("/proc/pressure/memory", "r");
    if(!memory_pressure) {
        perror("/proc/pressure/memory. Exiting.\n");
        fclose(memory_pressure);
        _exit(1);
    }

    float psi;

    if (EOF == fscanf(memory_pressure, "some avg10=%f", &psi)) {
        perror("Failed to read memory pressure values. Exiting.\n");
        fclose(memory_pressure);
        _exit(1);
    }

    fclose(memory_pressure);
    return psi;
}

free_mem_t poll_free_mem(void) {
    FILE * meminfo = fopen("/proc/meminfo", "r");
    if(!meminfo) {
        fprintf(stderr, "/proc/meminfo not found. Exiting.\n");
        fclose(meminfo);
        _exit(1);
    }

    char line[256];
    bool avail_mem_read = false;
    bool avail_swap_read = false;
    free_mem_t free_mem;

    while((!avail_mem_read || !avail_swap_read) && fgets(line, sizeof(line), meminfo))
    {
        int val;
        if(sscanf(line, "MemAvailable: %d kB", &val) == 1)
        {
            avail_mem_read = true;
            free_mem.available_mem_mib = (unsigned) val / 1024;
        }

        if(sscanf(line, "SwapFree: %d kB", &val) == 1)
        {
            avail_swap_read = true;
            free_mem.available_swap_mib = (unsigned) val / 1024;
        }
    }

    for (int i = 0; i < 100; i++) {
        putchar(' ');
    }

    if (!avail_swap_read || !avail_mem_read) {
        fprintf(stderr, "failed to read available system memory or swap amounts. Exiting.\n");
        fclose(meminfo);
        _exit(1);
    }

    fclose(meminfo);
    return free_mem;
}

int main(void) {
    time_t start, now;
    float time_left = 4.0;

    time(&start);

    while(time_left > 0.0) {
    	time(&now);
        time_left = 4.0 - difftime(now, start);

    	printf("\rmem-eater will start consuming system memory in: %.2f secs. Press Ctrl+C if you don't want that to happen.", time_left);
    	fflush(stdout);
    	usleep(20);
    }

    while(1)
    {
        free_mem_t free_mem = poll_free_mem();
        float psi = memory_pressure_some_avg_10();
        display(&free_mem, psi);
        void *m = malloc(1024*1024);
        memset(m,0,1024*1024);
    }


	return 0;
}
