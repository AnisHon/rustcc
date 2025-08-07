/* 多行注释：
   用于测试词法分析器的注释处理能力
*/

// 函数声明
int factorial(int n);

// 主函数
int main() {
    // 变量声明与初始化
    int x = 42;
    float y = 3.14e-2;
    char c = 'w';
    char str[] = "Hello, Lexer!\n";

    // 控制结构
    if (x > MAX_VALUE) {
        printf("Overflow\n");
    } else {
        for (int i = 0; i < x; i++) {
            y *= i + 1;
        }
    }

    // 函数调用
    int result = factorial(5);
    printf("Factorial: %d\n", result);

    // 特殊符号和运算符
    int *ptr = &x;
    int arr[5] = {1, 2, 3};
    int bitwise = x | 0xFF;

    return 0;
}

// 函数定义
int factorial(int n) {
    return (n <= 1) ? 1 : n * factorial(n - 1);
}