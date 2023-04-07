package rjvm;

public class InstanceOfArray {
    interface Intf1 {
    }

    static class C0 {
    }

    static class C1 implements Intf1 {
    }

    public static void main(String[] args) {
        checkInstanceOfInterfaces(new C0[0]);
        checkInstanceOfInterfaces(new C1[0]);
    }

    private static void checkInstanceOfInterfaces(Object[] v) {
        tempPrint(v instanceof Object[]);
        tempPrint(v instanceof C0[]);
        tempPrint(v instanceof C1[]);
        tempPrint(v instanceof Intf1[]);
    }

    private static native void tempPrint(boolean value);
}
