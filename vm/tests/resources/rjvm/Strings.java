package rjvm;

public class Strings {
    public static void main(String[] args) {
        sayHello("andrea");
    }

    private static void sayHello(String name) {
        tempPrint("Hello, " + name);
    }

    private static native void tempPrint(String value);
}
