void prints(const char *str);
void printc(const char sym);

extern void main()
{
	char *str = "C kernel loaded.";
	prints(str);
}

void prints(const char *str)
{
	for (int i = 0; str[i] != 0; i++)
	{
		printc(str[i]);
	}
}

void printc(const char sym)
{
	static int xpos = 0;
	volatile char *buffer = (volatile char *) 0xB8000;
	buffer += xpos;
	
	*buffer = sym;
	*(buffer+1) = 0x0F;
	xpos += 2;
}
