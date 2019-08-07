#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
int main(int argc, char * argv[]) {
    if (argc != 3) {
        printf("Usage: ts value port");
        return 1;
    }
    int magic = atoi(argv[1]);
    int port = atoi(argv[2]);
    __asm__ __volatile__("outl %0, %1"
    :
    : "a" ((uint32_t)magic),
      "d" ((uint16_t)port)
    : "memory");
}
