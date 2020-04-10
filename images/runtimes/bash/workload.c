#include <stdio.h>
#include <sys/socket.h>
#include <linux/vm_sockets.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
/*

struct WriteReq {
	char key[20];
	int value;
};

struct ReadReq {
	char key[20];	
};
*/

int main () {
	printf("hello pipi");
	int sock;
	struct sockaddr_vm sock_addr;
	int res;
	char read_req[32];
	char write_req[32];
	sock = socket(AF_VSOCK, SOCK_STREAM, 0);
	if (sock == -1) {
		printf("cannot create sock :(");
	} else {
		printf("created sock :)");
	}
	sock_addr.svm_family = AF_VSOCK;
	sock_addr.svm_reserved1 = 0;
	sock_addr.svm_port = 52;
	sock_addr.svm_cid = 2;
	/*res = bind(sock, (const struct sockaddr *)&sock_addr, sizeof(sock_addr));
	if (res == -1)
		printf("cannot bind :(");
	res = listen(sock, 1);
	if (res == -1)
		printf("cannot listen :(");
	*/
	res = connect(sock, (const struct sockaddr *)&sock_addr, sizeof(sock_addr));
	if (res == -1) {
		printf("cannot connect :(");
	} else {
		printf("connected :)");
	}
	//struct WriteReq write_req = {"pi", 3};
	//unsigned char *buffer = (char*)malloc(sizeof(write_req));
	//memcpy(buffer, (const unsigned char*) &write_req, sizeof(write_req));
	// 1 : create_dir
	// 2 : metadata
	// 3 : read
	// 4 : write
	// 5 : copy
	// 6 : remove_dir
	// 7 : remove_dir_all
	// 8 : remove_file
	// 9 : set_permissions
	char op[] = "create_dir";
	char key[] = "pi";
	char value[] = "3.14";
	//size_t payload_length = sizeof(op) + sizeof(key) +sizeof(value);
	char end[] = "\r";
	//memcpy(write_req, &payload_length, sizeof(payload_length));
	memcpy(write_req, &op, sizeof(op));
	memcpy(write_req + sizeof(op), &key, sizeof(key));
	memcpy(write_req + sizeof(op) + sizeof(key), &value, sizeof(value));
	memcpy(write_req + sizeof(op) + sizeof(key) + sizeof(value), &end, sizeof(end));
	write(sock, write_req, sizeof(op) + sizeof(key) + sizeof(value) + sizeof(end));
	//usleep(1000*1000);
	char op_read[] = "2";
	memcpy(read_req, &op_read, sizeof(op_read));
	memcpy(read_req + sizeof(op_read), &key, sizeof(key));
	memcpy(read_req + sizeof(op_read) + sizeof(key), &end, sizeof(end));
	write(sock, read_req, sizeof(op_read) + sizeof(key) + sizeof(end));
	char buffer[32];
	bzero(buffer, 32);
	read(sock, buffer, 32);
	printf("[C client] read value: %s", buffer);
	return 0;
}
