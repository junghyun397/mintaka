package renju.notation

final case class Pos(row: Int, col: Int) {

  def idx: Int = this.row * Renju.BOARD_WIDTH + this.col

  override def toString: String = f"${(col + 97).toChar}${row + 1}"

}

object Pos {

  @inline def idxToRow(idx: Int): Int = idx / Renju.BOARD_WIDTH
  @inline def idxToCol(idx: Int): Int = idx % Renju.BOARD_WIDTH

  @inline def rowColToIdx(row: Int, col: Int): Int = row * Renju.BOARD_WIDTH + col

  def fromCartesian(raw: String): Option[Pos] = for {
    row <- raw.drop(1).toIntOption
    col = raw.head
    pos <- this.fromCartesian(col, row)
  } yield pos

  def fromCartesian(rawCol: String, rawRow: Int): Option[Pos] = this.fromCartesian(rawCol.charAt(0), rawRow)

  def fromCartesian(rawCol: Char, rawRow: Int): Option[Pos] = {
    val col = rawCol.toUpper - 65
    val row = rawRow - 1

    Option.when(col >= 0 && row >= 0 && row < Renju.BOARD_WIDTH && col < Renju.BOARD_WIDTH) { new Pos(row, col) }
  }

  def fromIdx(idx: Int): Pos = new Pos(idxToRow(idx), idxToCol(idx))

}
