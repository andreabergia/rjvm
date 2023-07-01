package rjvm;

public class GarbageCollection {
    public static void main(String[] args) {
        // Gc roots can be objects or arrays
        AWrapperObject anObjectThatShouldNotBeDestroyed = new AWrapperObject(0);
        AWrapperObject[] anotherObjectAlive = new AWrapperObject[]{new AWrapperObject(-1)};

        int count = 10;
        ASmallObject[] anotherArray = new ASmallObject[count];
        for (int i = 1; i <= count; ++i) {
            // Trigger GC repeatedly
            new AWrapperObject(i);

            anotherArray[i - 1] = new ASmallObject(i);
        }

        tempPrint("checking references are still alive...");
        tempPrint(anObjectThatShouldNotBeDestroyed.getValue());
        tempPrint(anotherObjectAlive[0].getValue());
        tempPrint(anotherArray[0].value);
    }

    public static class AWrapperObject {
        private final int value;
        private final ASmallObject aSmallObject;
        private final ALargeObject aLargeObject = new ALargeObject();

        public AWrapperObject(int value) {
            this.value = value;
            this.aSmallObject = new ASmallObject(value);
            tempPrint(value);

            aLargeObject.oneMegabyteOfData[0] = value;
        }

        public long getValue() {
            return this.value + this.aSmallObject.value + this.aLargeObject.oneMegabyteOfData[0];
        }
    }

    public static class ASmallObject {
        private final long value;

        public ASmallObject(long value) {
            this.value = value;
        }
    }

    public static class ALargeObject {
        private final long[] oneMegabyteOfData = new long[1024 * 1024 / 8];
    }

    private static native void tempPrint(long value);
    private static native void tempPrint(String value);
}
