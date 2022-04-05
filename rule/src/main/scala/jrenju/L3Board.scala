package jrenju

import jrenju.notation.Opening

class L3Board(
  boardField: Array[Byte],
  val attackField: Array[AttackPoints],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening],
  val isEnd: Boolean,
) extends Board(boardField, moves, latestMove, opening) {

  def calculateDeepL3Board(): DeepL3Board = new DeepL3Board(
    this.boardField,
    this.attackField,
    this.moves,
    this.latestMove,
    this.opening,
    this.isEnd
  )

}

final class DeepL3Board(
  boardField: Array[Byte],
  attackField: Array[AttackPoints],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening],
  isEnd: Boolean,
) extends L3Board(boardField, attackField, moves, latestMove, opening, isEnd)
