package rjvm;

public class ControlFlow {
    public static void main(String[] args) {
        controlFlowInts();
        controlFlowObjects(new Object());
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

    private static native void tempPrint(int value);
}
