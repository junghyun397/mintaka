package jrenju.notation

final case class Pos(row: Int, col: Int) {

  def idx: Int = this.row * Renju.BOARD_WIDTH + this.col

  def unapply(arg: Pos): (Int, Int) = (this.col, this.row)

  def toCartesian: String = f"${(col + 97).toChar}${row + 1}"

}

object Pos {

  @inline def idxToRow(idx: Int): Int = idx / Renju.BOARD_WIDTH
  @inline def idxToCol(idx: Int): Int = idx % Renju.BOARD_WIDTH

  @inline def rowColToIdx(row: Int, col: Int): Int = row * Renju.BOARD_WIDTH + col

  def fromCartesian(rawRow: Int, rawCol: String): Option[Pos] = this.fromCartesian(rawRow, rawCol.charAt(0))

  def fromCartesian(rawRow: Int, rawCol: Char): Option[Pos] = {
    val col = rawCol.toUpper - 65
    val row = rawRow - 1
    if (col < 0 || col >= Renju.BOARD_WIDTH || row < 0 || row >= Renju.BOARD_WIDTH) Option.empty
    else Option(new Pos(row, col))
  }

  def fromIdx(idx: Int): Pos = new Pos(idxToRow(idx), idxToCol(idx))

}
