#include <stdio.h>
#include <sys/socket.h>
#include <linux/vm_sockets.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>

int main () {
	printf("hello pipi");
	/*
	 * Connect to VMM over vsock
	 *
	 * */
	int sock;
	struct sockaddr_vm sock_addr;
	int res;
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
	res = connect(sock, (const struct sockaddr *)&sock_addr, sizeof(sock_addr));
	if (res == -1) {
		printf("cannot connect :(");
	} else {
		printf("connected :)");
	}

	/*
	 * Send requests to VMM to manage VMM's local FS
	 * 
	 * - create a new directory
	 * - create a text file
	 * - read from the text file
	 * - copy to a new file
	 * - remove the whole directory
	 *
	 * */
	char req0[32];
	char op[] = "create_dir";
	char dir[] = "pidir";
	char end[] = "\r";
	memcpy(req0, &op, sizeof(op));
	memcpy(req0 + sizeof(op), &dir, sizeof(dir));
	memcpy(req0 + sizeof(op) + sizeof(dir), &end, sizeof(end));
	write(sock, req0, sizeof(op) + sizeof(dir) + sizeof(end));
	
	char req1[64];
	char op1[] = "write";
	char filename[] = "pidir/todo.txt";
	char body[] = "1. take out trash\n2. laundry\n3. call grandma\n";
	memcpy(req1, &op1, sizeof(op1));
	memcpy(req1 + sizeof(op1), &filename, sizeof(filename));
	memcpy(req1 + sizeof(op1) + sizeof(filename), &body, sizeof(body));
	memcpy(req1 + sizeof(op1) + sizeof(filename) + sizeof(body), &end, sizeof(end));
	write(sock, req1, sizeof(op1) + sizeof(filename) + sizeof(body) + sizeof(end));

	char req2[32];
	char op2[] = "read";
	memcpy(req2, &op2, sizeof(op2));
	memcpy(req2 + sizeof(op2), &filename, sizeof(filename));
	memcpy(req2 + sizeof(op2) + sizeof(filename), &end, sizeof(end));
	write(sock, req2, sizeof(op2) + sizeof(filename) + sizeof(end));
	char op2_buffer[500];
	//bzero(op2_buffer, 500);
	read(sock, op2_buffer, 500);
	printf("[C client] read value: %s", op2_buffer);

	char req3[32];
	char op3[] = "copy";
	char filename_cp[] = "pidir/todo-copy.txt";
	memcpy(req3, &op3, sizeof(op3));
	memcpy(req3 + sizeof(op3), &filename, sizeof(filename));
	memcpy(req3 + sizeof(op3) + sizeof(filename), &filename_cp, sizeof(filename_cp));
	memcpy(req3 + sizeof(op3) + sizeof(filename) + sizeof(filename_cp), &end, sizeof(end));
	write(sock, req3, sizeof(op3) + sizeof(filename) + sizeof(filename_cp) + sizeof(end));

	char req4[32];
	char op4[] = "remove_dir_all";
	memcpy(req4, &op4, sizeof(op4));
	memcpy(req4 + sizeof(op4), &dir, sizeof(dir));
	memcpy(req4 + sizeof(op4) + sizeof(dir), &end, sizeof(end));
	write(sock, req4, sizeof(op4) + sizeof(dir) + sizeof(end));

	return 0;
}
