#include <stdio.h>

#define MAX_PERSONS 1024
#define DEBUG


int main(){
  int personID = 1;
  int person2ID = personID + 1;

  {
    int personID = 2;
    personID +=1 ;
  }

  #ifdef DEBUG
  printf("WE ARE IN DEBUG MODE: %s:%d", __FILE__, __LINE__);
  #endif

  return 0;
}
