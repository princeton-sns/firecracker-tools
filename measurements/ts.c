#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
int main() {
	int magic = 123;
	int port = 0x03f0;
	__asm__ __volatile__("outl %0, %1"
        :
        : "a" ((uint32_t)magic),
          "d" ((uint16_t)port)
        : "memory");
}
