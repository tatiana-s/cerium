char id_char(char x)
{
    return x;
}

float id_float(float y)
{
    return y;
}

int id_int(int x)
{
    return x;
}

int main(void)
{
    id_char('2');
    id_float(2.2);
    id_int(22);
    return 0;
}