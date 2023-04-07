package rjvm;

public class InvokeInterface {
    interface Polygon {
        int area();
    }

    static class Rectangle implements Polygon {
        private final int width;
        private final int height;

        Rectangle(int width, int height) {
            this.width = width;
            this.height = height;
        }

        @Override
        public int area() {
            return width * height;
        }
    }

    static class Square implements Polygon {
        private final int side;

        Square(int side) {
            this.side = side;
        }

        @Override
        public int area() {
            return side * side;
        }
    }

    static class NotReallyASquare extends Square implements Polygon {
        NotReallyASquare(int side) {
            super(side);
        }

        @Override
        public int area() {
            return super.area() + 1;
        }
    }

    public static void main(String[] args) {
        printAreas(new Polygon[]{
                new Rectangle(3, 4),
                new Square(2),
                new NotReallyASquare(3),
        });
    }

    private static void printAreas(Polygon[] polygons) {
        for (Polygon p : polygons) {
            tempPrint(p.area());
        }
    }

    private static native void tempPrint(int value);
}
