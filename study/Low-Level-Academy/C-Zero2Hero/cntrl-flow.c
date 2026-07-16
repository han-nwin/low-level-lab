#include <stdio.h>

#define MAX_IDS 32

//function
int sum(int a, int b) {
  return a + b;
}
int main() {

  int temp;
  printf("What temperture is it? ");
  scanf("%d", &temp);

  if (temp >= 70) {
    printf("dang bruther, it's hot\n");
  } else if (temp >= 30 && temp < 70){
    printf("dang bruther, it's mild\n");
  } else {
    printf("dang bruther, it's cold\n");
  }

  //for loop
  int i = 0;
  for(i = 0; i < MAX_IDS; i++){
    //do something
  }
  //while loop
  i = 0;
  while(i < MAX_IDS){
    //do something
    i++;
  }
  //do while loop
  i = 0;
  do {
    //do something (it will do as least 1)
    i++;
  } while (i < MAX_IDS);


  printf("%d\n", sum(2,3));
  return 0;
}
