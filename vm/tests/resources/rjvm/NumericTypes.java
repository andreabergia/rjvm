package rjvm;

public class NumericTypes {
    public static void main(String[] args) {
        shortAndCharMath((short) 1, (char) 2);
        floatAndIntMath(1, 2.45f);
        longMath(1, 3);
    }

    private static void shortAndCharMath(short s, char c) {
        tempPrint(((short)(s + c)));
    }

    private static void floatAndIntMath(int i, float f) {
        tempPrint(i + f);
    }

    private static void longMath(int i, long l) {
        tempPrint(l - i);
    }

    private static native void tempPrint(int value);
    private static native void tempPrint(long value);
    private static native void tempPrint(float value);
}
