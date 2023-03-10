package rjvm;

public class SuperClasses {
    public static void main(String[] args) {
        DerivedClass d = new DerivedClass();
        tempPrint(d.someVirtualMethod());
    }

    private static native void tempPrint(int value);

    static class BaseClass {
        int someVirtualMethod() {
            return 1;
        }
    }

    static class DerivedClass extends BaseClass {
    }
}
