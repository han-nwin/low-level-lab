#include <stdio.h>
#include <unistd.h>
#include <stdbool.h>

#define MAX_IDS 32
#define MAX_EMPLPOYEES 100

//__atrribute__((__packed__)) part will create the same struct but ensure that
//the compiler won't add any special sauce in between the elements so we can
//ensure it's the same size on multiple system
struct employee_t {
  int id;
  char firstname[64];
  char lastname[64];
  float income;
  bool ismanager;

};

// Only assign 1 memory location for every elements.
// The size of the memory is of the largest element
union Data {
  int intValue;
  float floatValue;
  char charValue;
};

int main () {
  struct employee_t employee[MAX_EMPLPOYEES];

  union Data data;

  data.intValue = 10;
    printf("Int Value: %d\n", data.intValue);

    data.floatValue = 3.14;
    printf("Float Value: %.2f\n", data.floatValue);

    data.charValue = 'A';
    printf("Char Value: %c\n", data.charValue);

    // Notice that only the last value assigned (charValue) is correctly printed
    printf("\nAfter all assignments:\n");
    printf("Size of the location %ld\n", sizeof(data));
    printf("Int Value: %d\n", data.intValue);
    printf("Float Value: %.2f\n", data.floatValue);
    printf("Char Value: %c\n", data.charValue);




  return 0;
}
