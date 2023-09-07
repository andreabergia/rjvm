package rjvm;

public class ObjectArrays {
    public static void main(String[] args) {
        Square[] squares = new Square[]{
                new Square(1),
                new Square(2),
                null,
        };

        int totalArea = 0;
        for (int i = 0; i < squares.length; ++i) {
            if (squares[i] != null) {
                totalArea += squares[i].area();
            }
        }
        tempPrint(totalArea);
    }

    private static native void tempPrint(int value);

    public static final class Square {
        private final int side;

        public Square(int side) {
            this.side = side;
        }

        public int area() {
            return side * side;
        }
    }
}
