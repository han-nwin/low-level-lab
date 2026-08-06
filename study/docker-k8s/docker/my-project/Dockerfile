# Use Ubuntu as the base image
FROM ubuntu:24.04

# Install the tools needed to build the project
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /my-docker-app

# Copy the project into the container
COPY . .

# Configure the project
RUN cmake -S . -B build

# Build the project
RUN cmake --build build

# Run the executable
CMD ["./build/my_project"]
