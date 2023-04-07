package rjvm;

public class InstanceOf {
    interface Intf1 {}
    interface Intf2 extends Intf1 {}
    interface Intf3 extends Cloneable {}
    static class C1 {}
    static class C2 implements Intf1 {}
    static class C3 implements Intf2 {}
    static class C4 implements Intf3 {}
    static class C5 implements Intf3, Intf1 {}

    public static void main(String[] args) {
        tempPrint(new C1() instanceof Object);
        tempPrint(new C1() instanceof C1);

        checkInstanceOfInterfaces(new C1());
        checkInstanceOfInterfaces(new C2());
        checkInstanceOfInterfaces(new C3());
        checkInstanceOfInterfaces(new C4());
        checkInstanceOfInterfaces(new C5());
    }

    private static void checkInstanceOfInterfaces(Object v) {
        tempPrint(v instanceof Intf1);
        tempPrint(v instanceof Intf2);
        tempPrint(v instanceof Intf3);
        tempPrint(v instanceof Cloneable);
    }

    private static native void tempPrint(boolean value);
}
