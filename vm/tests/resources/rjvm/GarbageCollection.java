package rjvm;

public class GarbageCollection {
    public static void main(String[] args) {
        ALargeObject anObjectThatShouldNotBeDestroyed = new ALargeObject(0);
        for (int i = 1; i <= 100; ++i) {
            new ALargeObject(i);
        }
        tempPrint("still alive: " + anObjectThatShouldNotBeDestroyed.value);
    }

    public static class ALargeObject {
        private final long[] oneMegabyteOfData = new long[1024 * 1024 / 8];
        private final int value;

        public ALargeObject(int value) {
            this.value = value;
            tempPrint("allocated " + value);
        }
    }

    private static native void tempPrint(String value);
}
