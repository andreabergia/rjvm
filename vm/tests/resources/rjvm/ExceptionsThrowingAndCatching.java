package rjvm;

class ExceptionsThrowingAndCatching {
    public static class E1 extends Exception {}
    public static class E2 extends E1 {}

    public static void main(String[] args) {
        try {
            throw new E1();
        } catch (E1 e1) {
            tempPrint(1);
        }

        try {
            throw new E2();
        } catch (E1 e1) {
            tempPrint(2);
        }

        try {
            throw new E2();
        } catch (E2 e2) {
            tempPrint(3);
        } catch (E1 e1) {
            tempPrint(4);
        }

        try {
            throwE2();
        } catch (E2 e2) {
            tempPrint(5);
        }

        throwAndCatchE1();
    }

    private static void throwE2() throws E2 {
        throw new E2();
    }

    private static void throwAndCatchE1() {
        try {
            throw new E1();
        } catch (Exception e) {
            tempPrint(6);
        }
    }

    private static native void tempPrint(int value);
}