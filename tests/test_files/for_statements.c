//output:
//0
//1
//3
//6
//10
//3
//-1
//-1
//0
//1
//3
//4

int main() {
    int a = 3;
    int b = 0;
    for (int a = 1; b <= 10; b = b + (a++)) {
        printf(b);
    }
    printf(a);
    // separators
    printf(-1);
    printf(-1);
    // test continue in for loop
    for (int i = 0; i != 5; ++i) {
        if (i == 2) continue;
        printf(i);
    }
}
