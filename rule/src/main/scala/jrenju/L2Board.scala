package jrenju

import jrenju.notation.{Flag, Opening}

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

    pointsField.zipWithIndex foreach { particleIdx =>
      val points = particleIdx._1.black

      if (points.fiveInRow > 0)
        this.boardField(particleIdx._2) = Flag.FREE
      else if (this.boardField(particleIdx._2) == Flag.FORBIDDEN_6)
        this.boardField(particleIdx._2) = Flag.FORBIDDEN_6
      else if (points.four > 1)
        this.boardField(particleIdx._2) = Flag.FORBIDDEN_44
      else if (points.three > 1) {
        this.boardField(particleIdx._2) = Flag.FORBIDDEN_33
        di3ForbidFlag = true
      }

    }

    new L3Board(this.boardField, pointsField, this.moves, this.latestMove, this.opening, this.winner, di3ForbidFlag)
  }

}
