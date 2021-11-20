#include <stdio.h>

extern const char *messager();

int main() {
	printf("%s!\n", messager());
}