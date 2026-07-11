#include <arpa/inet.h>
#include <netinet/in.h>
#include <pthread.h>
#include <signal.h>
#include <stdint.h>
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

// used to check if the client is still connected
typedef struct {
  int socket_fd;
  int connected;
} client_ctx_t;

int read_client_message(int socket_fd) {
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

void *thread_read(void *arg) {
  client_ctx_t *client_ctx = (client_ctx_t *)arg;
  while (client_ctx->connected) {
    if (read_client_message(client_ctx->socket_fd) < 0) {
      printf("read_client_message failed\n");
      client_ctx->connected = 0;
      break;
    }
  }
  return NULL;
}

void *thread_write(void *arg) {
  client_ctx_t *client_ctx = (client_ctx_t *)arg;
  while (client_ctx->connected) {
    if (send_message(client_ctx->socket_fd, "Hello from server") < 0) {
      printf("send_message failed\n");
      client_ctx->connected = 0;
      break;
    }
    sleep(1);
  }
  return NULL;
}

void *thread_client(void *arg) {
  client_ctx_t *client_ctx = (client_ctx_t *)arg;
  pthread_t read_thread;
  pthread_create(&read_thread, NULL, thread_read, client_ctx);
  pthread_t write_thread;
  pthread_create(&write_thread, NULL, thread_write, client_ctx);

  pthread_join(read_thread, NULL);
  pthread_join(write_thread, NULL);
  close(client_ctx->socket_fd); // close socket
  free(client_ctx);
  printf("A client disconnected\n");
  return NULL;
}

int main() {

  // ignore SIGPIPE
  signal(SIGPIPE, SIG_IGN);
  // Initialize sock address in | need to cast to sockaddr later
  struct sockaddr_in serverInfo = {0}; // 0 out server info -> if not might fail at bind
  /**
   * memset(&serverInfo, 0, sizeof(serverInfo)); //more dynamic
   * */
  // Client socket
  struct sockaddr_in clientInfo = {0};
  /**
   * memset(&clientInfo, 0, sizeof(clientInfo)); //more dynamic
   * */

  int clientSize = sizeof(clientInfo);

  serverInfo.sin_family = AF_INET; // IPv4
  serverInfo.sin_addr.s_addr = INADDR_ANY;
  /** Set address using a string
   *if (inet_pton(AF_INET, "127.0.0.1", &server_address.sin_addr) <= 0) {
      perror("Invalid address / Address not supported");
      return -1;
    }
    serverInfo.sin_addr.s_addr = inet_addr("127.0.0.1");
    */
  serverInfo.sin_port = htons(PORT); // 0x12345678 0x87654321

  int fd = socket(AF_INET, SOCK_STREAM, 0); // IPv4, SOCK_STREAM
  if (fd == -1) {
    perror("Socket failed");
    return -1;
  }
  // bind
  if (bind(fd, (struct sockaddr *)&serverInfo, sizeof(serverInfo)) == -1) {
    perror("Bind Failed");
    return -1;
  }

  // listen
  if (listen(fd, 0) == -1) {
    perror("listen failed");
    return -1;
  }
  // converting ip address to human-readable form
  char ip_str[INET_ADDRSTRLEN];
  inet_ntop(AF_INET, &serverInfo.sin_addr, ip_str, INET_ADDRSTRLEN);

  printf("Server is listening to IP: %s port %d\n", ip_str, PORT);

  printf("Server waiting for connection...\n");
  while (1) {
    // accept
    int cfd = accept(fd, (struct sockaddr *)&clientInfo, (socklen_t *)&clientSize);
    if (cfd == -1) {
      perror("Accept failed");
      return -1;
    }
    printf("A client connected... socket_fd=%d\n", cfd);

    // create client thread
    pthread_t client_th;
    // create client context
    client_ctx_t *client_ctx = malloc(sizeof(client_ctx_t));
    client_ctx->socket_fd = cfd;
    client_ctx->connected = 1;
    pthread_create(&client_th, NULL, thread_client, client_ctx);
    pthread_detach(client_th);

    printf("\n");
    sleep(1);
  }
  return 0;
}
