#include <stdio.h>
#include <stdlib.h>

typedef enum {
  STATUS_GOOD,
  STATUS_BAD,
} status_t;


status_t foo(int **data, int len) {
  
  int *temp = NULL;
  temp = realloc(*data, len);

  if(temp == NULL) {
    *data = NULL;
    return STATUS_BAD;
  }
  
  *data = temp;
  return STATUS_GOOD;
}

int main() {
  int *first = malloc(64);
  if (STATUS_BAD == foo(&first, 128)) {
    printf("uh oh");
  }
  
  free(first);
  return 0;

}
