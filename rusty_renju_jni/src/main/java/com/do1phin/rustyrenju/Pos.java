package com.do1phin.rustyrenju;

public class Pos {

    public static int cartesianToIdx(int row, int col) {
        return row * Geometry::BOARD_WIDTH + col;
    }

    public static int idxToRow(int idx) {
        return idx / Geometry::BOARD_WIDTH;
    }

    public static int idxToCol(int idx) {
        return idx % Geometry::BOARD_WIDTH;
    }

    private byte value;

    public Pos(int idx) {
        this.value = (byte) idx;
    }

    public Pos(int row, int col) {
        final int idx = cartesianToIdx(row, col);
        this.value = (byte) idx;
    }

    public byte getValue() {
        return this.value;
    }

    public int getIdx() {
        return this.value & 0xFF;
    }

    public int getRow() {
        return idxToRow(this.getIdx());
    }

    public int getCol() {
        return idxToCol(this.getCol());
    }

}
