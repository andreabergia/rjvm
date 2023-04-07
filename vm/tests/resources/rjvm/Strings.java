package rjvm;

public class Strings {
    public static void main(String[] args) {
        sayHello("Andrea", 1985);
    }

    private static void sayHello(String name, int year) {
        tempPrint("Hello, " + name + ", you were born in " + year);
    }

    private static native void tempPrint(String value);
}
