package rjvm;

public class CheckCast {
    static class C1 {}

    static class C2 extends C1 {}

    public static void main(String[] args) {
        checkCast(new C2());
    }

    private static void checkCast(C1 c) {
        checkCasted((C2) c);
    }

    private static void checkCasted(C2 c) {
        tempPrint(c != null);
    }

    private static native void tempPrint(boolean value);
}
