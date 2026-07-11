#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <sys/socket.h>

#define PORT 8080


int main() {
    int server_fd, new_socket;
    struct sockaddr_in address;
    int addrlen = sizeof(address);
    char buffer[30000] = {0};

    // Create socket file descriptor
    if ((server_fd = socket(AF_INET, SOCK_STREAM, 0)) == 0) {
        perror("Socket failed");
        exit(EXIT_FAILURE);
    }

    // Define server address
    address.sin_family = AF_INET;
    address.sin_addr.s_addr = INADDR_ANY;
    address.sin_port = htons(PORT);

    // Bind the socket to the port
    if (bind(server_fd, (struct sockaddr *)&address, sizeof(address)) < 0) {
        perror("Bind failed");
        exit(EXIT_FAILURE);
    }

    // Listen for incoming connections
    if (listen(server_fd, 3) < 0) {
        perror("Listen failed");
        exit(EXIT_FAILURE);
    }
    printf("Server is running on http://127.0.0.1:%d\n", PORT);
    
    
    //HANDLE Client Connection
    while (1) {
        printf("\nWaiting for a connection...\n");

        if ((new_socket = accept(server_fd, (struct sockaddr *)&address, (socklen_t*)&addrlen)) < 0) {
            perror("Accept failed");
            exit(EXIT_FAILURE);
        }

        read(new_socket, buffer, 30000);
        printf("Received request:\n%s\n", buffer);

        // Prepare HTTP response
        char *response = "HTTP/1.1 200 OK\nContent-Type: text/html\nContent-Length: 37\n\n<html><body>Hello, World!</body></html>";

        // Send response
        write(new_socket, response, strlen(response));
        printf("Response sent.\n");

        // Close the socket for this client
        close(new_socket);
    }
    
    return 0;
}

