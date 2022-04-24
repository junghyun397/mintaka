package jrenju

import jrenju.notation.{Flag, Opening, Renju}

final class L2Board(
  boardField: Array[Byte],
  pointsField: Array[PointsPair],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening],
  override val winner: Byte,
) extends Board(boardField, pointsField, moves, latestMove, opening) with EvaluatedBoard {

  def calculateL3Board(): L3Board = {
    var di3ForbidFlag = false

    for (idx <- 0 until Renju.BOARD_LENGTH) {
      val points = this.pointsField(idx).black

      if (points.fiveInRow > 0)
        this.boardField(idx) = Flag.FREE
      else if (this.boardField(idx) == Flag.FORBIDDEN_6)
        this.boardField(idx) = Flag.FORBIDDEN_6
      else if (points.four > 1)
        this.boardField(idx) = Flag.FORBIDDEN_44
      else if (points.three > 1) {
        this.boardField(idx) = Flag.FORBIDDEN_33
        di3ForbidFlag = true
      }
    }

    new L3Board(this.boardField, pointsField, this.moves, this.latestMove, this.opening, this.winner, di3ForbidFlag)
  }

}
