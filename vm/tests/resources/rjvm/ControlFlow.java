package rjvm;

public class ControlFlow {
    public static void main(String[] args) {
        controlFlowInts();
        controlFlowObjects(new Object());
        controlFlowLongFloatDouble(1, 1, 1);
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

    private static void controlFlowObjects(Object a) {
        if (a != null) {
            tempPrint(42);
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

    private static native void tempPrint(int value);
}
