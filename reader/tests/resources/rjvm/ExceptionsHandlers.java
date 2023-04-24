package rjvm;

class ExceptionsHandlers {
    void foo() {
    }

    void bar() throws Exception {
    }

    void test() throws Exception {
        try {
            bar();
        } finally {
            foo();
        }

        try {
            bar();
        } catch (IllegalStateException e) {
            bar();
        }
    }
}