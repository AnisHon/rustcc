/* 1. 基本语句测试 */
{
    printf("\n=== 基本语句测试 ===\n");

    // 表达式语句
    a = 10;
    a++;  // 后缀自增
    ++a;  // 前缀自增
    printf("1.1 表达式语句: a = %d\n", a);

    // 空语句
    ; // 这是一个空语句
    printf("1.2 空语句测试通过\n");

    // 复合语句(块)
    {
        b = 20;
        printf("1.3 复合语句: b = %d\n", b);
    }
    // printf("b = %d\n", b); // 错误: b不可见
}

/* 2. 控制流语句测试 */
{
    printf("\n=== 控制流语句测试 ===\n");

    // if-else 语句
    x = 5;
    if (x > 10) {
        printf("2.1 if: x > 10\n");
    } else if (x > 0) {
        printf("2.1 if-else: 0 < x <= 10\n");
    } else {
        printf("2.1 if-else: x <= 0\n");
    }

    // switch 语句
    switch (x) {
        case 1:
            printf("2.2 switch: case 1\n");
            break;
        case 5:
            printf("2.2 switch: case 5\n");
            // 故意不加break测试fall-through
        case 6:
            printf("2.2 switch: case 5 or 6\n");
            break;
        default:
            printf("2.2 switch: default\n");
    }

    // while 循环
    count = 0;
    while (count < 3) {
        printf("2.3 while: count = %d\n", count);
        count++;
    }

    // do-while 循环
    count = 0;
    do {
        printf("2.4 do-while: count = %d\n", count);
        count++;
    } while (count < 3);

    // for 循环
    for (int i = 0; i < 3; i++) {
        printf("2.5 for: i = %d\n", i);
    }

    // for循环中的复杂表达式
    for (int i = 0, j = 10; i < j; i++, j--) {
        printf("2.6 复杂for: i=%d, j=%d\n", i, j);
    }
}

/* 3. 跳转语句测试 */
{
    printf("\n=== 跳转语句测试 ===\n");

    // goto 语句
    n = 0;
    LOOP_LABEL:
    if (n < 3) {
        printf("3.1 goto: n = %d\n", n);
        n++;
        goto LOOP_LABEL;
    }

    // break 语句
    for (i = 0; i < 5; i++) {
        if (i == 3) break;
        printf("3.2 break: i = %d\n", i);
    }

    // continue 语句
    for (i = 0; i < 5; i++) {
        if (i % 2 == 0) continue;
        printf("3.3 continue: i = %d\n", i);
    }

    // return 语句
    printf("3.4 return: 测试将在函数末尾执行\n");
    return; // 提前返回
    printf("这行不会执行\n");
}

/* 4. 声明语句测试 */
//{
//    printf("\n=== 声明语句测试 ===\n");
//
//    // 基本变量声明
//    a = 10;
//    PI = 3.14159;
//    counter = 0;
//    printf("4.1 基本声明: a=%d, PI=%.2f, counter=%d\n", a, PI, counter);
//
//    // 数组声明
//    arr = {1, 2, 3};
//    printf("4.2 数组声明: arr[2]=%d, arr[4]=%d\n", arr[2], arr[4]);
//
//    // 结构体声明
//    struct Point {
//        int x;
//        int y;
//    } p1 = { .x = 10, .y = 20 };
//    printf("4.3 结构体声明: p1.x=%d, p1.y=%d\n", p1.x, p1.y);
//
//    // 联合体声明
//    union Data {
//        int i;
//        float f;
//    } data = { .i = 100 };
//    printf("4.4 联合体声明: data.i=%d\n", data.i);
//
//    // 枚举声明
//    enum Color { RED, GREEN, BLUE } color = GREEN;
//    printf("4.5 枚举声明: color=%d\n", color);
//}

/* 5. 标签语句测试 */
{
    printf("\n=== 标签语句测试 ===\n");

    // case 标签 (已在switch中测试)

    // default 标签 (已在switch中测试)

    // 普通标签
    i = 0;
    MY_LABEL:
    if (i < 2) {
        printf("5.1 普通标签: i=%d\n", i);
        i++;
        goto MY_LABEL;
    }
}

/* 6. 复杂语句组合测试 */
{
    printf("\n=== 语句组合测试 ===\n");

    // 嵌套控制流
    for (i = 0; i < 3; i++) {
        printf("6.1 外层循环: i=%d\n", i);
        for (j = 0; j < 2; j++) {
            if (j == 1) {
                printf("6.1 内层if: j=%d\n", j);
            }
        }
    }

    // 带初始化的if语句(C99)
    if (x = 5; x > 3) {
        printf("6.2 带初始化的if: x=%d\n", x);
    }

    // switch中的复杂case
    c = 'A';
    switch (c) {
        case 'A' ... 'Z':
            printf("6.3 switch范围case: 大写字母\n");
            break;
        case 'a' ... 'z':
            printf("6.3 switch范围case: 小写字母\n");
            break;
        default:
            printf("6.3 switch范围case: 其他字符\n");
    }
}

/* 7. 特殊语句测试 */
{
    printf("\n=== 特殊语句测试 ===\n");

    // asm语句 (编译器相关)
    // printf("7.1 asm语句:\n");
    // __asm__("nop");

    // _Static_assert (C11)
    printf("7.2 静态断言:\n");
    _Static_assert(sizeof(int) >= 2, "int必须至少2字节");

    // 带属性语句 (C23)
    // printf("7.3 带属性语句:\n");
    // [[deprecated]] int old_var = 0;
}

{
    test_basic_statements();
    test_control_flow();
    test_jump_statements();
    test_declaration_statements();
    test_labeled_statements();
    test_statement_combinations();
    test_special_statements();

    printf("\n=== 语句测试完成 ===\n");
    return 0;
}