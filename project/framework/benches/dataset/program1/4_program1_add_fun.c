int id_int(int x)
{
    return x;
}

float id_float(float y)
{
    return y;
}

char id_char(char x)
{
    return x;
}

int main(void)
{
    id_int(22);
    id_float(2.2);
    id_char('2');
    return 0;
}