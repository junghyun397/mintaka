import javax.annotation.Nullable;

public class Pattern {

    private static native void destroy(long pattern_ptr);

    private static native boolean isForbidden(long pattern_ptr);
    private static native int forbiddenKind(long pattern_ptr);
    private static native long getUnit(long pattern_ptr, isBlack: boolean);

    private final long patternPtr;

    public boolean isForbidden() {
        return isForbidden(this.patternPtr);
    }

    @Nullable
    public ForbiddenKind getForbiddenKind() {
        final int forbiddenKind = this.forbiddenKind(this.patternPtr);
        if (forbiddenKind == 0) return null;
        else return forbiddenKind;
    }

    public PatternUnit getUnit(color: Color) {
        return getUnit(this.patternPtr, color == Color.BLACK);
    }

    @Override
    protected void finalize() throws Throwable {
        destroy(this.patternPtr);
        super.finalize();
    }

}

public class PatternUnit {

    private static native void destroy(long pattern_ptr);

    private static native int countOpenThree(long pattern_unit_ptr);
    private static native int countCloseThree(long pattern_unit_ptr);
    private static native int countOpenFour(long pattern_unit_ptr);
    private static native int countClosedFour(long pattern_unit_ptr);
    private static native int countTotalFour(long pattern_unit_ptr);
    private static native int countFive(long pattern_unit_ptr);

    private final long patternUnitPtr;

    public int countOpenThree() {
        countOpenThree(this.patternUnitPtr);
    }

    public int countCloseThree() {
        countCloseThree(this.patternUnitPtr);
    }

    public int countOpenFour() {
        countOpenFour(this.patternUnitPtr);
    }

    public int countClosedFour() {
        countClosedFour(this.patternUnitPtr);
    }

    public int countTotalFour() {
        countTotalFour(this.patternUnitPtr);
    }

    public int countFive() {
        countFive(this.patternUnitPtr);
    }

    @Override
    protected void finalize() throws Throwable {
        destroy(this.patternUnitPtr);
        super.finalize();
    }

}
