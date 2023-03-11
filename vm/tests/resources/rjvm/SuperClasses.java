package rjvm;

public class SuperClasses {
    public static void main(String[] args) {
        rjvm.SuperClasses.DerivedClass d = new rjvm.SuperClasses.DerivedClass(2);
        d.setBaseValue(1);
        tempPrint(d.sum());
    }

    private static native void tempPrint(int value);

    static abstract class BaseClass {
        protected int baseValue = 0;

        void setBaseValue(int value) {
            this.baseValue = value;
        }

        int sum() {
            return baseValue + derivedClassValue();
        }

        protected abstract int derivedClassValue();
    }

    static class DerivedClass extends rjvm.SuperClasses.BaseClass {
        private final int derivedValue;

        public DerivedClass(int derivedValue) {
            this.derivedValue = derivedValue;
        }

        @Override
        int sum() {
            return 1 + super.sum();
        }

        @Override
        protected int derivedClassValue() {
            return this.derivedValue;
        }
    }
}
