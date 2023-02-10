package rjvm;

public class SimpleMain {
    public static void main(String[] args) {
        Generator g = new Generator(0, 3);
        tempPrint(g.next());
        tempPrint(g.next());
    }

    private static native void tempPrint(int value);

    static class Generator {
        private int curr;
        private final int inc;

        Generator(int start, int inc) {
            this.curr = start;
            this.inc = inc;
        }

        public int next() {
            this.curr += this.inc;
            return this.curr;
        }
    }
}
