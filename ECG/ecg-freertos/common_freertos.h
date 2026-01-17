
/* Add all the necessary #include and #define headers */
// #include "stream_buffer.h"
void __attribute__((noinline, used, optimize("O0"))) syz_builtin0() {
	printf("calling syz_builtin0");
}
void __attribute__((noinline, used, optimize("O0"))) syz_builtin1() {printf("calling syz_builtin1");}
void __attribute__((noinline, used, optimize("O0"))) syz_builtin2() {printf("calling syz_builtin2");}
void __attribute__((noinline, used, optimize("O0"))) syz_builtin3() {printf("calling syz_builtin3");}
void __attribute__((noinline, used, optimize("O0"))) syz_builtin4() {printf("calling syz_builtin4");}
void __attribute__((noinline, used, optimize("O0"))) syz_builtin5() {printf("calling syz_builtin5");}

void __attribute__((noinline, used, optimize("O0"))) test() {

	int a = 0;
	for (int i = 0; i < 100; ++i) {
		a+= i;
	}
}

static void syz_mycall(int a, int b) {
	int c = a + b;
	test();
	for (int i = 0; i < 100; ++i) {
		a+= i;
	}
	
}


static void syz_mycall1(int c, int a, int b) {
	printf("calling syz_mycall1\n");
}

static void syz_mycall2(int n1, int n2) {
	int i, gcd;

    printf("Enter two integers: ");

    for(i=1; i <= n1 && i <= n2; ++i)
    {
        // Checks if i is factor of both integers
        if(n1%i==0 && n2%i==0)
            gcd = i;
    }


}

static void syz_mycall3(int n) {
  printf("Enter an integer: ");

  for (int i = 1; i <= 10; ++i) {
    printf("%d * %d = %d \n", n, i, n * i);
  }
	int a[10] = {0};
	for (int i = 0; i < 1000; ++i){
		a[i] =  10;
	}
	// buffer overflow 
	if (n < 100) {
		printf("n is less than 100\n");
		char buffer[10];
		for (int i = 0; i < 12; ++i) {
			buffer[i] = 'A';
		}
		printf("Buffer contains: %s\n", buffer);
	}
	// use after free
	if (n < 1000) {
		printf("n is less than 1000\n");
		char* buffer = malloc(10);
		strcpy(buffer, "Hello");
		free(buffer);  // Freeing the buffer
		printf("Buffer still contains: %s\n", buffer);  
	}
	// double free
	if (n < 10000) {
		printf("n is less than 10000\n");
		char* buffer = malloc(10);
		free(buffer);  // First free
		free(buffer);  // Second free
	}
	// Dangling ptr
	if (n < 20000) {
	    char* buffer1 = malloc(10);
		char* buffer2;
		strcpy(buffer1, "Hello");
		free(buffer1);  
		buffer2 = buffer1; 
	}

}
