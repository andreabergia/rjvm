package rjvm;

public class NumericTypes {
    public static void main(String[] args) {
        shortAndCharMath((short) 1, (char) 2);
        floatAndIntMath(1, 2.45f);
        longMath(1, 3);
        doubleMath(1, 3.45);
    }

    private static void shortAndCharMath(short s, char c) {
        tempPrint(((short) (s + c)));
    }

    private static void floatAndIntMath(int i, float f) {
        tempPrint(i + f);
        tempPrint((int)(i + f));
        tempPrint((long)(i + f));
        tempPrint((double)(i + f));
    }

    private static void longMath(int i, long l) {
        tempPrint(l - i);
        tempPrint((int)(l - i));
        tempPrint((float)(l - i));
        tempPrint((double)(l - i));
    }

    private static void doubleMath(int i, double d) {
        tempPrint(i + d);
        tempPrint((int)(i + d));
        tempPrint((float)(i + d));
        tempPrint((long)(i + d));
    }

    private static native void tempPrint(int value);

    private static native void tempPrint(long value);

    private static native void tempPrint(float value);

    private static native void tempPrint(double value);
}
