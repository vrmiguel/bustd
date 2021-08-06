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

void free_mem_print(free_mem_t * mem) {
    printf("\rFree system memory: %d MiB. Free swap: %d MiB", mem->available_mem_mib, mem->available_swap_mib);
    fflush(stdout);
}


free_mem_t poll_free_mem(void)
{
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
    float time_left = 10.0;

    time(&start);

    while(time_left > 0.0) {
    	time(&now);
    	time_left = 10.0 - difftime(now, start);

    	printf("\rmem-eater will start consuming system memory in: %.2f secs. Press Ctrl+C if you don't want that to happen.", time_left);
    	fflush(stdout);
    	usleep(20);
    }

    free_mem_t free_mem = poll_free_mem();
    free_mem_print(&free_mem);

    while(1)
    {
        free_mem_t free_mem = poll_free_mem();
        free_mem_print(&free_mem);
        void *m = malloc(1024*1024);
        memset(m,0,1024*1024);
    }


	return 0;
}