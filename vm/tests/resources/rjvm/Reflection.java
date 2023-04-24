package rjvm;

public class Reflection {
    private int value = 2;

    public static void main(String[] args) throws ClassNotFoundException, InstantiationException, IllegalAccessException {
        Class<Reflection> theClass = (Class<Reflection>) Class.forName("rjvm.Reflection");
        Reflection reflection = theClass.newInstance();
        tempPrint(reflection.value);
    }

    private static native void tempPrint(int value);
}
