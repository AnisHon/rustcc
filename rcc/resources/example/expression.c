
    // 1. 嵌套三元运算符
    x = 10, y = 20, z = 30;
    (x > y) ? (y > z ? y : z) : (x > z ? x : z);
    result = (x > y) ? (y > z ? y : z) : (x > z ? x : z);
    printf("1. 嵌套三元运算: %d\n", result);

    // 2. 复杂位运算表达式
    flags = 0x00F0;
    mask = 0x0F0F;
    (flags & mask) | ((~flags) & (mask << 1)) ^ 0xAAAA;
    combined = (flags & mask) | ((~flags) & (mask << 1)) ^ 0xAAAA;
    printf("2. 复杂位运算: 0x%04X\n", combined & 0xFFFF);

    // 3. 混合类型表达式
    d = 3.14159;
    n = 10;
    c = 'A';
    mixed = (d * n) + (c / (d - floor(d))) - (n % (int)(d * 10));
    printf("3. 混合类型计算: %.4f\n", mixed);

    // 4. 指针算术与数组访问
//    arr = {1, 3, 5, 7, 9, 11};
    *ptr = arr + 2;
    arr_val = *(ptr + 1) + ptr[-1] * (*ptr % 4);
    printf("4. 指针算术: %d\n", arr_val);

    // 5. 复合赋值与副作用
    a = 5, b = 7;
    a += (b += 3) - (a -= 2);
    printf("5. 复合赋值链: a=%d, b=%d\n", a, b);

    // 6. 函数调用嵌套表达式
    printf("6. 函数嵌套: %.2f\n", sqrt(pow(sin(0.5), 2) + pow(cos(0.5), 2)));

