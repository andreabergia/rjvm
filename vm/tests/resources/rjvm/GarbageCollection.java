package rjvm;

public class GarbageCollection {
    public static void main(String[] args) {
        AWrapperObject anObjectThatShouldNotBeDestroyed = new AWrapperObject(0);
        AWrapperObject[] anotherObjectAlive = new AWrapperObject[]{new AWrapperObject(-1)};

        for (int i = 1; i <= 10; ++i) {
            new AWrapperObject(i);
        }
//        tempPrint("still alive: " + anObjectThatShouldNotBeDestroyed.value);
        tempPrint(anObjectThatShouldNotBeDestroyed.getValue());
        tempPrint(anotherObjectAlive[0].getValue());
    }

    public static class AWrapperObject {
        private final int value;
        private final ASmallObject aSmallObject;
        private final ALargeObject aLargeObject = new ALargeObject();

        public AWrapperObject(int value) {
            this.value = value;
            this.aSmallObject = new ASmallObject(value);
//            tempPrint("allocated " + value);

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
}
