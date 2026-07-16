int main() {
  char c; // 1 byte: 8 bits
  short s;// 2 bytes: 16 bits
  int i; // 4 bytes: 32 bits
  long long int l; // 8 bytes, 64 bits
  
  unsigned int x = 4;
  int sx = (int)(x);

  unsigned int bigx = 0xfffffefe;
  short kindabig = (short)bigx; // may lose information
  
  //upcasting
  short short2 = -1; // 0xffff
  int wasashort = (int)short2; //0xfffffff cast the sign

  

}
