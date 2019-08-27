#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
int main(int argc, char * argv[]) {
	int magic = strtoul(argv[1], NULL, 0);
	int port = strtoul(argv[2], NULL, 0);
	__asm__ __volatile__("outl %0, %1"
        :
        : "a" ((uint32_t)magic),
          "d" ((uint16_t)port)
        : "memory");
}
