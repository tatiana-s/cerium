int id_int(int x)
{
    return x;
}

float id_float(int y)
{
    return y;
}

int main(void)
{
    id_int(22);
    id_float(2.2);
    return 0;
}