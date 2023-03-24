package rjvm;

public class NumericArrays {
    public static void main(String[] args) {
        playWithArrayOfBooleans(new boolean[2]);
        playWithArrayOfBytes(new byte[]{0x01, 0x02});
        playWithArrayOfChars(new char[]{'a', 'b'});
        playWithArrayOfShorts(new short[]{1, 2});
    }

    private static void playWithArrayOfBooleans(boolean[] array) {
        array[0] = true;
        tempPrint(array[0] || array[1]);
        tempPrint(array.length);
    }

    private static void playWithArrayOfBytes(byte[] array) {
        tempPrint(array[0] | array[1]);
    }

    private static void playWithArrayOfChars(char[] array) {
        tempPrint(array[0] > array[1] ? array[0] : array[1]);
    }

    private static void playWithArrayOfShorts(short[] array) {
        tempPrint(array[0] - array[1]);
    }

    private static native void tempPrint(boolean value);

    private static native void tempPrint(int value);
}
