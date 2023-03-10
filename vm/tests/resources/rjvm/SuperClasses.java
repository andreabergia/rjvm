package rjvm;

public class SuperClasses {
    public static void main(String[] args) {
        DerivedClass d = new DerivedClass(2);
        d.setBaseValue(1);
        tempPrint(d.sum());
    }

    private static native void tempPrint(int value);

    static class BaseClass {
        protected int baseValue = 0;

        void setBaseValue(int value) {
            this.baseValue = value;
        }
    }

    static class DerivedClass extends BaseClass {
        private final int derivedValue;

        public DerivedClass(int derivedValue) {
            this.derivedValue = derivedValue;
        }

        public int sum() {
            return super.baseValue + this.derivedValue;
        }
    }
}
