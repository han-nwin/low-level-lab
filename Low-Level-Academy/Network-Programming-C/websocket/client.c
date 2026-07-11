#include <arpa/inet.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

#define PORT 5555
#define BUFF_SIZE 4096

// Defined proto type to be used
typedef enum {
  PROTOCOL_TCP_HAN,
  PROGOCOL_TCP_HAN_NGUYEN,

} proto_type_e;

// TLV (type length value) system
typedef struct {
  proto_type_e type;
  unsigned short len;

} proto_header_t;

int read_server_message(int socket_fd) {
  // define read buffer
  char buff[BUFF_SIZE] = {0};

  // NOTE: the header and payload sent in as 1 single buffer

  // define header buff
  proto_header_t *header = (proto_header_t *)buff;

  // == Read header ==
  // we wanna read the entire header length. 1 read() doesn't guarantee that
  // hence the while loop
  size_t read_header_size = 0;
  while (read_header_size < sizeof(proto_header_t)) {
    ssize_t n = read(socket_fd, buff + read_header_size, sizeof(proto_header_t) - read_header_size);
    if (n == 0) {
      // client disconnected
      return -1;
    }
    if (n < 0) {
      perror("read error");
      return -1;
    }
    read_header_size += n;
  }

  // Read header data to the header buff
  header->type = ntohs(header->type); // This is payload type
  if (header->type != PROTOCOL_TCP_HAN && header->type != PROGOCOL_TCP_HAN_NGUYEN) {
    printf("Wrong prototype %d\n", header->type);
    return -1;
  }
  header->len = ntohs(header->len); // This is payload len

  // == Read payload ==
  if (header->len >= BUFF_SIZE - sizeof(proto_header_t)) {
    printf("payload too big\n");
    return -1;
  }

  // read to buff at the offset sizeof(header size) -> the payload
  // we wanna read the entire payload length. 1 read() doesn't guarantee that
  // hence the while loop
  size_t read_payload_size = 0;
  while (read_payload_size < header->len) {
    ssize_t n = read(socket_fd, buff + sizeof(proto_header_t) + read_payload_size, header->len - read_payload_size);
    if (n == 0) {
      // client disconnected
      return -1;
    }
    if (n < 0) {
      perror("read error");
      return -1;
    }
    read_payload_size += n;
  }

  // Access the payload
  char *payload = buff + sizeof(proto_header_t);
  // make it string data with terminate null byte
  payload[header->len] = '\0';
  printf("type=%d, len=%d, message=%s\n", header->type, header->len, payload);
  return 0;
}

int send_message(int socket_fd, char *message) {
  // define header buff
  proto_header_t header = {0};
  header.type = htons(PROTOCOL_TCP_HAN);
  header.len = htons(strlen(message));

  // define write buffer
  char buff[BUFF_SIZE] = {0};

  // write header
  memcpy(buff, &header, sizeof(proto_header_t));
  // write payload
  memcpy(buff + sizeof(proto_header_t), message, strlen(message));

  // write to socket
  if (write(socket_fd, buff, sizeof(proto_header_t) + strlen(message)) < 0) {
    perror("write error");
    return -1;
  }
  return 0;
}

void *thread_read(void *socket_fd) {
  while (1) {
    if (read_server_message(*(int *)socket_fd) < 0) {
      printf("read_server_message failed\n");
      break;
    }
  }
  return NULL;
}

void *thread_write(void *socket_fd) {
  while (1) {
    int sock_fd = *(int *)socket_fd;
    char message[BUFF_SIZE] = {0};

    snprintf(message, sizeof(message), "Hello from client, my socket_fd is %d on my end", sock_fd);
    if (send_message(sock_fd, message) < 0) {
      printf("send_message failed\n");
      break;
    }
    sleep(1);
  }
  return NULL;
}
int main(int argc, char **argv) {

  if (argc != 2) {
    printf("Usage: %s <ip of the host>\n", argv[0]);
    return 1;
  }

  struct sockaddr_in serverInfo = {0};
  /**
   * memset(&serverInfo, 0, sizeof(serverInfo)); //more dynamic
   * */

  serverInfo.sin_family = AF_INET; // IPv4
  // Set address using a string
  if (inet_pton(AF_INET, argv[1], &serverInfo.sin_addr) <= 0) {
    perror("Invalid address / Address not supported");
    return -1;
  }
  serverInfo.sin_port = htons(PORT);

  // Create socket
  int fd = socket(AF_INET, SOCK_STREAM, 0); // Ipv4
  if (fd == -1) {
    perror("Socket failed");
    return -1;
  }

  // Connect
  if (connect(fd, (struct sockaddr *)&serverInfo, sizeof(serverInfo)) == -1) {
    perror("Connect failed");
    close(fd);
    return -1;
  }
  printf("Connected successfully!\n");

  pthread_t read_thread;
  pthread_create(&read_thread, NULL, thread_read, &fd);
  pthread_t write_thread;
  pthread_create(&write_thread, NULL, thread_write, &fd);

  pthread_join(read_thread, NULL);
  pthread_join(write_thread, NULL);
  close(fd); // close socket

  return 0;
}
