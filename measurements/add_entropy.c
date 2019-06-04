#include <stdio.h>
#include <stdlib.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <sys/ioctl.h>
#include <linux/random.h>

#define BUFSIZE 256
typedef struct rand_pool_info rand_pool_t;

int main(){
	int res;
	int fd = open("/dev/random", O_RDONLY);
	printf("add entropy to /dev/random\n");
	rand_pool_t rand_data;
	rand_data.entropy_count = 1024;
	rand_data.buf_size = BUFSIZE;

	res = ioctl(fd, RNDADDENTROPY, &rand_data);

}
