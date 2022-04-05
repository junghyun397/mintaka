package jrenju

import jrenju.notation.Opening

final class L2Board(
  boardField: Array[Byte],
  val attackField: Array[AttackPoints],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening],
  val isEnd: Boolean
) extends Board(boardField, moves, latestMove, opening) {

  def calculateL3Board(): L3Board = new L3Board(this.boardField, this.attackField, this.moves, this.latestMove, this.opening, this.isEnd)

}
