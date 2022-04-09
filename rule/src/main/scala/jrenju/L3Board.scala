package jrenju

import jrenju.notation.Opening

class L3Board(
  boardField: Array[Byte],
  val attackField: Array[(AttackPoints, AttackPoints)],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening],
  val winner: Byte,
) extends Board(boardField, moves, latestMove, opening) {

  def calculateDeepL3Board(): DeepL3Board = new DeepL3Board(
    this.boardField,
    this.attackField,
    this.moves,
    this.latestMove,
    this.opening,
    this.winner,
  )

}

final class DeepL3Board(
  boardField: Array[Byte],
  attackField: Array[(AttackPoints, AttackPoints)],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening],
  winner: Byte,
) extends L3Board(boardField, attackField, moves, latestMove, opening, winner)
