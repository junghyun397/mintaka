package com.do1phin.rustyrenju;

public class Board {

    private static native void drop(long board_ptr);

    private static native long fromString(String source);
    private static native long fromMoves(byte[] moves);
    private static native long fromEachColorMoves(byte[] black_moves, byte[] white_moves, int player);
    private static native String toString(long board_ptr);
    private static native String toDetailedString(long board_ptr);

    private static native int getStones(long board_ptr);
    private static native long getHashKey(long board_ptr);

    private static native long set(long board_ptr, byte pos);
    private static native long unset(long board_ptr, byte pos);
    private static native long pass(long board_ptr);

    private static native void setMut(long board_ptr, byte pos);
    private static native void unsetMut(long board_ptr, byte pos);
    private static native void passMut(long board_ptr);

    private static native boolean getPlayerIsBlack(long board_ptr);

    private static native boolean isForbidden(long board_ptr, byte pos);
    private static native boolean getForbiddenKind(long board_ptr, byte pos);

    private static native boolean countOpenThree(long board_ptr, byte pos, boolean is_black);
    private static native boolean countCloseThree(long board_ptr, byte pos, boolean is_black);
    private static native boolean countOpenFour(long board_ptr, byte pos, boolean is_black);
    private static native boolean countClosedFour(long board_ptr, byte pos, boolean is_black);
    private static native boolean countTotalFour(long board_ptr, byte pos, boolean is_black);
    private static native boolean countFive(long board_ptr, byte pos, boolean is_black);

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

    public boolean getPlayerIsBlack() {
        return getPlayerIsBlack(this.boardPtr);
    }

    public boolean isForbidden(Pos pos) {
        isForbidden(this.boardPtr, pos.getRawValue());
    }

    public int countOpenThree(Pos pos, Color color) {
        countOpenThree(this.boardPtr, pos.getRawValue(), color == Color.BLACK);
    }

    public int countCloseThree(Pos pos, Color color) {
        countCloseThree(this.boardPtr, pos.getRawValue(), color == Color.BLACK);
    }

    public int countOpenFour(Pos pos, Color color) {
        countOpenFour(this.boardPtr, pos.getRawValue(), color == Color.BLACK);
    }

    public int countClosedFour(Pos pos, Color color) {
        countClosedFour(this.boardPtr, pos.getRawValue(), color == Color.BLACK);
    }

    public int countTotalFour(Pos pos, Color color) {
        countTotalFour(this.boardPtr, pos.getRawValue(), color == Color.BLACK);
    }

    public int countFive(Pos pos, Color color) {
        countFive(this.boardPtr, pos.getRawValue(), color == Color.BLACK);
    }

    @Override
    public String toString() {
        return toString(this.boardPtr);
    }

    public String toDetailedString() {
        return toDetailedString(this.boardPtr);
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
    protected void drop() throws Throwable {
        drop(this.boardPtr);
        super.finalize();
    }

}
