package rjvm;

public class ControlFlow {
    public static void main(String[] args) {
        Object o = new Object();
        Object[] arr = new Object[]{o};

        controlFlowInts();
        controlFlowObjects(o, o, new Object());
        controlFlowLongFloatDouble(1, 1, 1);
        controlFlowArrays(arr, arr, new Object[2]);
    }

    private static void controlFlowInts() {
        int x = 1;
        while (x < 100) {
            if (x % 2 == 0) {
                x = x * 3 + 1;
            } else {
                x += 1;
            }
        }
        tempPrint(x);
    }

    private static void controlFlowObjects(Object a, Object b, Object c) {
        if (a != null) {
            tempPrint(42);
        }
        if (a == b) {
            tempPrint(43);
        }
        if (a == c) {
            tempPrint(44);
        }
    }

    private static void controlFlowLongFloatDouble(long l, float f, double d) {
        if (l > 0) {
            tempPrint(1);
        }
        if (f > 0) {
            tempPrint(1);
        }
        if (d > 0) {
            tempPrint(1);
        }
    }

    private static void controlFlowArrays(Object[] a, Object[] b, Object[] c) {
        if (a != null) {
            tempPrint(51);
        }
        if (a == b) {
            tempPrint(52);
        }
        if (a == c) {
            tempPrint(53);
        }
    }

    private static native void tempPrint(int value);
}
