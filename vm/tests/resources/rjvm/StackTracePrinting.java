package rjvm;

class StackTracePrinting {
    public static void main(String[] args) {
        Throwable ex = new Exception();
        StackTraceElement[] stackTrace = ex.getStackTrace();
        for (StackTraceElement element : stackTrace) {
            tempPrint(
                    element.getClassName() + "::" + element.getMethodName() + " - " +
                            element.getFileName() + ":" + element.getLineNumber());
        }
    }

    private static native void tempPrint(String value);
}