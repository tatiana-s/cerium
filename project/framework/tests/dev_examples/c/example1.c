#include <stdio.h>
#include <stdbool.h>

int add(int a, int b)
{
    return a + b;
}

int main(void)
{
    printf("%i", add('a', 'b'));
    return 0;
}
