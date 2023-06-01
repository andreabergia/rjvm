package rjvm;

public class NumericArrays {
    public static void main(String[] args) {
        playWithArrayOfBooleans(new boolean[2]);
        playWithArrayOfBytes(new byte[]{0x01, 0x02});
        playWithArrayOfChars(new char[]{'a', 'b'});
        playWithArrayOfShorts(new short[]{1, 2});
        playWithArrayOfInts(new int[]{3, 4});
        playWithArrayOfLongs(new long[]{4, 2});
        playWithArrayOfFloats(new float[]{1.2f, 0.2f});
        playWithArrayOfDoubles(new double[]{0.0, 3.3});
        copyArrays();
    }

    private static void playWithArrayOfBooleans(boolean[] array) {
        array[0] = true;
        tempPrint(array[0] || array[1]);
        tempPrint(array.length);
    }

    private static void playWithArrayOfBytes(byte[] array) {
        tempPrint(array[0] | array[1]);
        tempPrint(array.length);
    }

    private static void playWithArrayOfChars(char[] array) {
        tempPrint(array[0] > array[1] ? array[0] : array[1]);
        tempPrint(array.length);
    }

    private static void playWithArrayOfShorts(short[] array) {
        tempPrint(array[0] - array[1]);
        tempPrint(array.length);
    }

    private static void playWithArrayOfInts(int[] array) {
        tempPrint(array[0] * array[1]);
        tempPrint(array.length);
    }

    private static void playWithArrayOfLongs(long[] array) {
        tempPrint(array[0] / array[1]);
        tempPrint(array.length);
    }

    private static void playWithArrayOfFloats(float[] array) {
        tempPrint(array[0] + array[1]);
        tempPrint(array.length);
    }

    private static void playWithArrayOfDoubles(double[] array) {
        tempPrint(array[0] * array[1]);
        tempPrint(array.length);
    }

    private static void copyArrays() {
        int[] source = new int[]{1, 2, 3, 4};
        int[] dest = new int[]{0, 0, 0, 0, 0};
        System.arraycopy(source, 1, dest, 1, 3);
        tempPrint(dest[0]);
        tempPrint(dest[1]);
        tempPrint(dest[2]);
        tempPrint(dest[3]);
        tempPrint(dest[4]);
        tempPrint(dest.length);
    }

    private static native void tempPrint(boolean value);

    private static native void tempPrint(int value);

    private static native void tempPrint(long value);

    private static native void tempPrint(float value);

    private static native void tempPrint(double value);
}
