#include <stdio.h>
#include <fcntl.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <stdlib.h>
#include <unistd.h>

struct database_header_t {
  unsigned short version;
  unsigned short employees;
  unsigned int fileSize;
};

int main(int argc, char **argv) {
  
  struct database_header_t head = {0};
  struct stat dbStat = {0};

  if (argc != 2) {
    printf("Usage: %s <filename>\n", argv[0]);
    return -1;
  }

  int fd = open(argv[1], /**O_CREAT |*/ O_RDWR, 0644);
  if (fd == -1) {
    perror("open");
    return -1;
  }

  char *buff = "Hello there";
  //write(fd, buff, sizeof(&buff));


  // Ensure we are at the start of the file
  if (lseek(fd, 0, SEEK_SET) == -1) {
    perror("lseek");
    close(fd);
    return -1;
  }
  //Read the file INTO a struct (doesn't have to be char *buff)
  if (read(fd, &head, sizeof(head)) != sizeof(head)) {
    perror("read");
    close(fd);
    return -1;
  }

  printf("DB version: %d\n", head.version);
  printf("Employees: %d\n", head.employees);
  printf("DB file size: %d\n", head.fileSize);

  if (fstat(fd, &dbStat) < 0) {
    perror("fstat");
    close(fd);
    return -1;
  }

  printf("DB file length, reported by stat: %lu\n", dbStat.st_size);

  if (dbStat.st_size != head.fileSize) {
    printf("GET OUTTA HERE");
    close(fd);
    return -1;
  }

  close(fd);

  return 0;

}

