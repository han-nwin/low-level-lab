#include <stdio.h>

#define MAX_IDS 32

int main() {
  int ids[MAX_IDS] = {0, 1, 2};

  printf("%d\n", ids[0]);

  ids[3] = 0x41; //hex type 
  
  printf("%d\n", ids[3]);


  char mystr[] = {'\x48', '\x65', '\x6C', '\x6C', '\x6F', '\x00'};
  char *myotherstr = "helllo"; //no need null terminator
  printf("%s\n",mystr);

}
