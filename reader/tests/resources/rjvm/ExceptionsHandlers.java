package rjvm;

class ExceptionsHandlers {
    void foo() {
    }

    void bar() throws IllegalArgumentException, IllegalStateException {
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