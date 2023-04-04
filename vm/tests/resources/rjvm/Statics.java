package rjvm;

public class Statics {
    public static void main(String[] args) {
        MyObject myObject1 = new MyObject(1);
        MyObject myObject2 = new MyObject(2);
        tempPrint(myObject1.weirdFunForTesting());
        tempPrint(myObject2.weirdFunForTesting());
    }

    private static final class MyObject {
        private static int nextId = 1;
        private final int value;
        private final int id;

        public MyObject(int value) {
            this.value = value;
            this.id = MyObject.nextId++;
        }

        public int weirdFunForTesting() {
            return MyObject.nextId * 100 + this.id * 10 + this.value;
        }
    }

    private static native void tempPrint(int value);
}
