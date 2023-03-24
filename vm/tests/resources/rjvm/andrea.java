package rjvm;

public class andrea {
    public static void main(String[] args) {
        logicalShifts(4);
    }

    private static void logicalShifts(int i) {
        tempPrint(i >> 2);
        tempPrint((-i) >>> 2);
        tempPrint(i << 1);
    }

    private static void tempPrint(int value) {
        System.out.println(value);
    }
}
