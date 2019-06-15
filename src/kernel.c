void main () {
   char * vga = (char *) 0xb8000 ;
   *vga = 'X';
   *(vga+1) = 0x0F;
   for (;;);
}

