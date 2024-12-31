class Board {
    private static native long fromString(String source);
    private static native long fromMoves(byte[] moves);
    private static native long fromMoves(byte[] blackMoves, byte[] whiteMoves);
    private static native String toString(long ptr);

    private static native long set(long board_ptr, byte pos);
    private static native long unset(long board_ptr, byte pos);
    private static native long pass(long board_ptr, byte pos);

    private static native void setMut(long board_ptr, byte pos);
    private static native void unsetMut(long board_ptr, byte pos);
    private static native void passMut(long board_ptr, byte pos);

    private static native long getPattern(long board_ptr, byte pos);

    private static native boolean getPlayerIsBlack(long board_ptr);
}

class Pattern {
    private static native boolean isForbidden(long pattern_ptr);
    private static native int forbiddenKind(long pattern_ptr);
}

class PatternUnit {
    private static native int countOpenThree(long pattern_unit_ptr);
    private static native int countOpenFour(long pattern_unit_ptr);
    private static native int countCloseFour(long pattern_unit_ptr);
    private static native int countClosedFour(long pattern_unit_ptr);
    private static native int countTotalFour(long pattern_unit_ptr);
    private static native int countFive(long pattern_unit_ptr);
}
