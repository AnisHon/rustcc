//int i = 10;
//int *i = 20;
//const int a, * const b, *const *c = 10;

int (*func)(double);

// func1    =     fn (double, ...) -> int
// func2    =     fn (const char*, ...) -> *func1
// arr      =     [func2;10]
// a        =     * const volatile arr
//int (*(*(* const volatile a)[10])(const char*, ...))(double, ...);

struct fuck_you {
    int fuck_me;
    int (*(*(* const volatile fuck_him)[10])(const char*, ...))(double, ...);
    struct {} fuck_everything;
}
