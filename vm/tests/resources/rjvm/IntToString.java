package rjvm;

public class IntToString {
    public static void main(String[] args) {
        checkToStringMatches(1, "1");
    }

    static void checkToStringMatches(long value, String expected) {
        String actual = Long.toString(value);
        if (!actual.equals(expected)) {
            String message = "Expected " + expected + ", got " + actual;
            tempPrint(message);
            throw new AssertionError(message);
        }
    }

    private static native void tempPrint(String value);
}
