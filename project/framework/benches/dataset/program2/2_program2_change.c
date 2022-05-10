int problem1(void)
{
    int t = 0;
    int sum = 0;
    while (t)
    {
        t = t - 1;
        int p = 0;
        int sum = 0;

        int N = 1000;
        p = (N - 1) / 3;
        sum = ((3 * p * (p + 1)) / 2);

        p = (N - 1) / 5;
        sum = sum + ((5 * p * (p + 1)) / 2);

        p = (N - 1) / 15;
        sum = sum - ((15 * p * (p + 1)) / 2);
    }
    return sum;
}

int problem2(void)
{
    int n = 0;
    int sum = 0;
    int i = 1;
    int j = 2;
    int temp = 0;

    while (j <= n)
    {
        if (j == 0)
        {
            sum = j + 1;
        }
        temp = i;
        i = j;
        j = temp + i;
    }
    return sum;
}

int problem1_2(void)
{
    int t = 0;
    int sum = 0;
    while (t)
    {
        t = t - 1;
        int p = 0;
        int sum = 0;

        int N = 1000;
        p = (N - 1) / 3;
        sum = ((3 * p * (p + 1)) / 2);

        p = (N - 1) / 5;
        sum = sum + ((5 * p * (p + 1)) / 2);

        p = (N - 1) / 15;
        sum = sum - ((15 * p * (p + 1)) / 2);
    }
    return sum;
}

int problem2_2(void)
{
    int n = 0;
    int sum = 0;
    int i = 1;
    int j = 2;
    int temp = 0;

    while (j <= n)
    {
        if (j == 0)
        {
            sum = j + 1;
        }
        temp = i;
        i = j;
        j = temp + i;
    }
    return sum;
}

int problem1_3(void)
{
    int t = 0;
    int sum = 0;
    while (t)
    {
        t = t - 1;
        int p = 0;
        int sum = 0;

        int N = 1000;
        p = (N - 1) / 3;
        sum = ((3 * p * (p + 1)) / 2);

        p = (N - 1) / 5;
        sum = sum + ((5 * p * (p + 1)) / 2);

        p = (N - 1) / 15;
        sum = sum - ((15 * p * (p + 1)) / 2);
    }
    return sum;
}

int problem2_3(void)
{
    int n = 0;
    int sum = 0;
    int i = 1;
    int j = 2;
    int temp = 0;

    while (j <= n)
    {
        if (j == 0)
        {
            sum = j + 1;
        }
        temp = i;
        i = j;
        j = temp + i;
    }
    return sum;
}

int problem1_4(void)
{
    int t = 0;
    float sum = 0;
    while (t)
    {
        t = t - 1;
        int p = 0;
        int sum = 0;

        int N = 1000;
        p = (N - 1) / 3;
        sum = ((3 * p * (p + 1)) / 2);

        p = (N - 1) / 5;
        sum = sum + ((5 * p * (p + 1)) / 2);

        p = (N - 1) / 15;
        sum = sum - ((15 * p * (p + 1)) / 2);
    }
    return sum;
}

int problem2_4(void)
{
    int n = 0;
    int sum = 0;
    int i = 1;
    int j = 2;
    int temp = 0;

    while (j <= n)
    {
        if (j == 0)
        {
            sum = j + 1;
        }
        temp = i;
        i = j;
        j = temp + i;
    }
    return sum;
}