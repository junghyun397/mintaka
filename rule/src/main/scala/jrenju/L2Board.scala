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
      if (points.open3.sum > 1) {
        points.open3 = Points.empty
        this.boardField(particleIdx._2) = Flag.FORBIDDEN_33
        di3ForbidFlag = true
      }
      if (points.closed4.sum + points.open4.sum > 1) {
        points.closed4 = Points.empty
        points.open4 = Points.empty
        this.boardField(particleIdx._2) == Flag.FORBIDDEN_44
      }
      if (points.five.sum > 0) {
        points.open3 = Points.empty
        points.closed4 = Points.empty
        points.open4 = Points.empty
        this.boardField(particleIdx._2) = Flag.FREE
      }
      if (this.boardField(particleIdx._2) == Flag.FORBIDDEN_6) {
        points.open3 = Points.empty
        points.closed4 = Points.empty
        points.open4 = Points.empty
      }
    }

    new L3Board(this.boardField, pointsField, this.moves, this.latestMove, this.opening, this.winner, di3ForbidFlag)
  }

}
