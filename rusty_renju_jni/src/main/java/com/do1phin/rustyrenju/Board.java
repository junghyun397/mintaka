public class Board {

    private static native void destroy(long board_ptr);

    private static native long fromString(String source);
    private static native long fromMoves(byte[] moves);
    private static native long fromEachColorMoves(byte[] black_moves, byte[] white_moves, int player);
    private static native String toString(long board_ptr);

    private static native int getStones(long board_ptr);
    private static native long getHashKey(long board_ptr);

    private static native long set(long board_ptr, byte pos);
    private static native long unset(long board_ptr, byte pos);
    private static native long pass(long board_ptr);

    private static native void setMut(long board_ptr, byte pos);
    private static native void unsetMut(long board_ptr, byte pos);
    private static native void passMut(long board_ptr);

    private static native long getPattern(long board_ptr, byte pos);

    private static native boolean getPlayerIsBlack(long board_ptr);

    private final long boardPtr;

    private Board(long boardPtr) {
        this.boardPtr = boardPtr;
    }

    public int getStones() {
        return getStones(this.boardPtr);
    }

    public long getHashKey() {
        return getHashKey(this.boardPtr);
    }

    public Board set(Pos pos) {
        final newPtr = set(this.boardPtr, pos.getRawValue());
        return new Board(newPtr);
    }

    public Board unset(Pos pos) {
        final newPtr = unset(this.boardPtr, pos.getRawValue());
        return new Board(newPtr);
    }

    public Board pass() {
        final newPtr = pass(this.boardPtr);
        return new Board(newPtr);
    }

    public void setMut(Pos pos) {
        setMut(this.boardPtr, pos.getRawValue());
    }

    public void unsetMut(Pos pos) {
        unsetMut(this.boardPtr, pos.getRawValue());
    }

    public void passMut(Pos pos) {
        passMut(this.boardPtr);
    }

    public Pattern getPattern(Pos pos) {
        final patternPtr = getPattern(this.boardPtr, pos.getRawValue());
        return new Pattern(patternPtr);
    }

    public boolean getPlayerIsBlack() {
        return getPlayerIsBlack(this.boardPtr);
    }

    @Override
    public int toString() {
        return toString(this.boardPtr);
    }

    @Override
    public int hashCode() {
        return (int) this.getHashKey();
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        final Board board = (Board) obj;
        return this.boardPtr == board.boardPtr
            || this.getHashKey(this.boardPtr) == board.getHashKey(this.boardPtr);
    }

    @Override
    protected void finalize() throws Throwable {
        destroy(this.boardPtr);
        super.finalize();
    }

}
