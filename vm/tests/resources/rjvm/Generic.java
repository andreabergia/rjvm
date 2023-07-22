package rjvm;

import java.util.ArrayList;
import java.util.List;

public class Generic {
    public static void main(String[] args) {
        List<String> strings = new ArrayList<String>(10);
        strings.add("hey");
        strings.add("hackernews");

        for (String s : strings) {
            tempPrint(s);
        }
    }

    private static native void tempPrint(String value);
}
