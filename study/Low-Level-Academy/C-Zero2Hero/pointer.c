#include <stdio.h>
#include <stdlib.h>

struct employee_t {
  int id;
  int income;
  char *name;
};

int initialize_employee(struct employee_t *e) {
  static int numEmployees = 0;// Static value to keep track of number of employees through out the program
  numEmployees++;
  e->id = 0;
  e->income = 0;
  e->name = "No Name";

  return numEmployees;
}

void swap( int *a, int *b) {
    int temp;
    temp = *a;
    *a = *b;
    *b = temp;
}

int main() {
  int x = 10, y = 20;
  printf("%d %d\n", x, y);
  swap(&x, &y);
  printf("%d %d\n", x, y);

  int i = 3;
  int *pX = &i;
  printf("%p\n", pX);
  printf("%d\n", *pX);

  int num_emp = 4;
  struct employee_t *employees = malloc(sizeof(struct employee_t)*num_emp);
  if (employees == NULL) {
    printf("The allocator failed\n");
    return -1;
  }

  initialize_employee(&employees[0]);
  
  printf("%d\n", employees[0].income);

  free(employees);
  employees = NULL;



  return 0;
}
