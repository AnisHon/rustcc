#define COUNT 3

typedef unsigned long size_t;

struct point {
    int x;
    int y;
};

_Static_assert(COUNT > 0, "COUNT must be positive");
_Thread_local _Atomic int completed;

static inline int dot(const struct point *left, const struct point *right)
{
    return left->x * right->x + left->y * right->y;
}

int main(void)
{
    struct point points[COUNT] = {
        [0] = { .x = 1, .y = 2 },
        [1] = { .x = 3, .y = 4 },
        [2] = (struct point){ 5, 6 },
    };
    int result = dot(&points[0], &points[1]);
    return _Generic(result, int: result, default: 0);
}
