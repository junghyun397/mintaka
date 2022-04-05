package jrenju.notation

import jrenju.rule.Renju

final case class Pos(row: Int, col: Int) {

  def idx: Int = this.col * Renju.BOARD_WIDTH + this.row

  def unapply(arg: Pos): (Int, Int) = (this.row, this.col)

}

object Pos {

  def fromCartesian(rawRow: Char, rawCol: Int): Option[Pos] = {
    val row = rawRow.toUpper - 65
    val col = rawCol - 1
    if (row < 0 || row >= Renju.BOARD_WIDTH || col < 0 || col >= Renju.BOARD_WIDTH) Option.empty
    else Option.apply(new Pos(row, col))
  }

  def fromIdx(idx: Int): Pos = new Pos(idx % Renju.BOARD_WIDTH, idx / Renju.BOARD_WIDTH)

}
