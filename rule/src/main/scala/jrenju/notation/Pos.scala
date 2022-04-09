package jrenju.notation

import jrenju.rule.Renju

final case class Pos(row: Int, col: Int) {

  def idx: Int = this.col * Renju.BOARD_WIDTH + this.row

  def unapply(arg: Pos): (Int, Int) = (this.row, this.col)

  def toCartesian: String = f"${(row + 97).toChar}${col + 1}"

}

object Pos {

  @inline def idxToRow(idx: Int): Int = idx % Renju.BOARD_WIDTH
  @inline def idxToCol(idx: Int): Int = idx / Renju.BOARD_WIDTH

  @inline def rowColToIdx(row: Int, col: Int): Int = col * Renju.BOARD_WIDTH + row

  def fromCartesian(rawRow: String, rawCol: Int): Option[Pos] = this.fromCartesian(rawRow.charAt(0), rawCol)

  def fromCartesian(rawRow: Char, rawCol: Int): Option[Pos] = {
    val row = rawRow.toUpper - 65
    val col = rawCol - 1
    if (row < 0 || row >= Renju.BOARD_WIDTH || col < 0 || col >= Renju.BOARD_WIDTH) Option.empty
    else Option.apply(new Pos(row, col))
  }

  def fromIdx(idx: Int): Pos = new Pos(idxToRow(idx), idxToCol(idx))

}
