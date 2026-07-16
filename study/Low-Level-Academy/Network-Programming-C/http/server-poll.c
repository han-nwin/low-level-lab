#include <stdio.h>
#include <sys/select.h>
#include <arpa/inet.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <unistd.h>
#include <stdlib.h>
#include <string.h>
#include <poll.h>

#define MAX_CLIENTS 1024
#define PORT 8080
#define BUFF_SIZE 4096

typedef enum {
  STATE_NEW,
  STATE_CONNECTED,
  STATE_DISCONNECTED,
} state_e;

//Structure to hold client state
typedef struct {
  int fd;
  state_e state;
  char buffer[4096];
} clientstate_t;


clientstate_t clientStates[MAX_CLIENTS];

//Initialize client array
void init_client() {
  for (int i = 0; i < MAX_CLIENTS; i++) {
    clientStates[i].fd = -1;
    clientStates[i].state = STATE_NEW;
    memset(clientStates[i].buffer, '\0', BUFF_SIZE);
  }
}


//Find slot for new client
int find_free_slot() {
  for (int i = 0; i < MAX_CLIENTS; i++) {
    if (clientStates[i].fd == -1) {
      return i; //return the index of the free slot
    }
  }
  return -1; //full
}

//Find slot by fd (for poll)
int find_slot_by_fd (int fd) {
  for (int i = 0; i < MAX_CLIENTS; i++) {
    if (clientStates[i].fd == fd) {
      return i;
    }
  }
  return -1; //not found
}


//Main function
int main() {
  
  //Initialize and reset client array
  init_client();
  
  int num_fds; //track number of fd
  int freeSlot; //track free slot
  int opt = 1;

  //Initialize sock address in | need to cast to sockaddr later
  struct sockaddr_in server_address; 
  struct sockaddr_in client_address; 
  socklen_t client_size = sizeof(client_address);

  //Initialize pollfd structure array
  struct pollfd fds[MAX_CLIENTS + 1];
  
  //Initialize memory for serverInfo and clientInfo
  memset(&server_address, 0, sizeof(server_address)); //more dynamic
  memset(&client_address, 0, sizeof(client_address)); //more dynamic

  //Initialize sets of read and write
  fd_set read_fds;
  fd_set write_fds;

  //Connection fd Initialize
  int conn_fd;

  //Set connection protocol and port
  server_address.sin_family = AF_INET; //IPv4
  server_address.sin_addr.s_addr = INADDR_ANY;
  /** Set address using a string
   *if (inet_pton(AF_INET, "127.0.0.1", &server_address.sin_addr) <= 0) {
      perror("Invalid address / Address not supported");
      return -1;
    }   
    serverInfo.sin_addr.s_addr = inet_addr("127.0.0.1");
    */
  server_address.sin_port = htons(PORT); // 0x12345678 0x87654321


  //Open a socket
  int listen_fd = socket(AF_INET, SOCK_STREAM, 0); // IPv4, SOCK_STREAM

  if(listen_fd == -1) {
    perror("Socket failed");
    return -1;
  }


  //bind
  if (bind(listen_fd, (struct sockaddr *)&server_address, sizeof(server_address)) == -1){
    perror("Bind Failed");
    return -1;
  }


  //listen. Backlog 10 connections
  if (listen(listen_fd, 10) == -1) {
    perror("listen failed");
    return -1;
  }

  //converting ip address to human-readable form
  char ip_str[INET_ADDRSTRLEN];
  inet_ntop(AF_INET, &server_address.sin_addr, ip_str, INET_ADDRSTRLEN);
  printf("Server is listening to IP: %s port %d\n", ip_str, PORT);

  
  memset(fds, 0, sizeof(fds));
  fds[0].fd = listen_fd;
  fds[0].events = POLLIN;
  num_fds = 1;


  while (1) {

    int ii = 1;
    for (int i = 0; i < MAX_CLIENTS; i++) {
      if (clientStates[i].fd != -1) {
        fds[ii].fd = clientStates[i].fd; //Offset by 1 for listen_fd
        fds[ii].events = POLLIN;
        ii++;
      }
    }

    //Wait for an even on one of the sockets
    int n_events = poll(fds, num_fds, -1); // -1 means no timeout
    if (n_events == -1) {
      perror("poll");
      exit(EXIT_FAILURE);
    }

    //Check for new connections
    if (fds[0].revents & POLLIN) {
      if ((conn_fd = accept(listen_fd, (struct sockaddr *)&client_address, &client_size)) == -1) {
        perror("accept");
        continue;
      }

      printf("New connection from %s:%d\n", 
          inet_ntoa(client_address.sin_addr), ntohs(client_address.sin_port));

      freeSlot = find_free_slot();
      if (freeSlot == -1) {
        printf("Server full: Closing new connection \n");
        close(conn_fd);
      } else {
        clientStates[freeSlot].fd = conn_fd;
        clientStates[freeSlot].state = STATE_CONNECTED;
        num_fds++;
        printf("Slot %d has fd %d\n", freeSlot, clientStates[freeSlot].fd);
      }
      n_events--;
    }

    //Check each clietn for read/write activity
    for (int i = 1; i <= num_fds && n_events > 0; i++) {
      if (fds[i].revents & POLLIN) {
        n_events--;

        int fd = fds[i].fd;
        int slot = find_slot_by_fd(fd);
        ssize_t bytes_read = read(fd, &clientStates[slot].buffer, sizeof(clientStates[slot].buffer));
        if (bytes_read <= 0) {
          //connection closed or error
          close(fd);
          if (slot == -1) {
            printf("Tried to close fd that doesn't exist?\n");
          } else {
              clientStates[slot].fd = -1; //free up the slot
              clientStates[slot].state = STATE_DISCONNECTED;
              printf("Client disconnected or error\n");
              num_fds--;
          }
        } else {
            printf("Received data from client: %s\n", clientStates[slot].buffer);
        }

      }
    }
  }
  //return 0;
}
