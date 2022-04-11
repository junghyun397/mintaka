package jrenju

import jrenju.notation.{Flag, Opening}

class L3Board(
  boardField: Array[Byte],
  pointsField: Array[PointsPair],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening],
  override val winner: Byte,
  private val hasDi3Forbid: Boolean,
) extends Board(boardField, pointsField, moves, latestMove, opening) with EvaluatedBoard {

  private def isNegativeForbidden(idx: Int): Boolean = false // TODO

  def calculateDeepL3Board(): L3Board = {
    if (this.hasDi3Forbid) {
      this.boardField.zipWithIndex.foreach { indexedFlag =>
        if (indexedFlag._1 == Flag.FORBIDDEN_33 && this.isNegativeForbidden(indexedFlag._2))
          this.boardField(indexedFlag._2) = Flag.FREE
      }
      new DeepL3Board(
        this.boardField,
        this.moves,
        this.latestMove,
        this.opening,
        this.pointsField,
        this.winner,
        this.hasDi3Forbid,
      )
    } else
      this
  }

}

final class DeepL3Board(
  boardField: Array[Byte],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening],
  pointsField: Array[PointsPair],
  winner: Byte,
  hasDi3Forbid: Boolean,
) extends L3Board(boardField, pointsField, moves, latestMove, opening, winner, hasDi3Forbid)
