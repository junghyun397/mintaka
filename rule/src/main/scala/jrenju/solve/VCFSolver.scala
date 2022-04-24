package jrenju.solve

import jrenju.L3Board
import jrenju.notation.{Flag, Renju}

object VCFSolver {

  def findVCFSequenceBlack(board: L3Board, parents: Seq[Int]): Seq[Int] = {
    for (idx <- 0 until Renju.BOARD_LENGTH) {
      val points = board.pointsField(idx).black

      if (points.closedFour == 1) {
        if (points.three == 1)
          return parents.appended(idx)

        val companion = board.recoverClosed4Companion(points.closed4.indexOf(1).toByte, idx, _.black)

        if (board.pointsField(companion).white.four < 1) {
          val thenBoard = board
            .makeMove(idx)
            .calculateL2Board()
            .makeMove(companion)
            .calculateL2Board()
            .calculateL3Board()
            .calculateDeepL3Board()

          val solution = this.findVCFSequenceBlack(thenBoard, parents.appended(idx).appended(companion))

          if (solution.nonEmpty)
            return solution
        }
      }
    }

    Seq.empty
  }

  def findVCFSequenceWhite(board: L3Board, parents: Seq[Int]): Seq[Int] = {
    for (idx <- 0 until Renju.BOARD_LENGTH) {
      val points = board.pointsField(idx).white
      if (points.closed4.sum == 1) {
        if (points.three > 1 || points.four > 1)
          return parents.appended(idx)

        val companion = board.recoverClosed4Companion(points.closed4.indexOf(1).toByte, idx, _.white)

        if (Flag.isForbid(board.boardField(companion)))
          return parents.appended(idx)
        else if (board.pointsField(companion).black.four < 1) {
          val thenBoard = board
            .makeMove(idx)
            .calculateL2Board()
            .makeMove(companion)
            .calculateL2Board()
            .calculateL3Board()
            .calculateDeepL3Board()

          val solution = this.findVCFSequenceWhite(thenBoard, parents.appended(idx).appended(companion))

          if (solution.nonEmpty)
            return solution
        }
      }
    }

    Seq.empty
  }


  implicit class VCFFinder(val board: L3Board) {

    def findVCFSequence(): Seq[Int] =
      if (this.board.moves % 2 == 0)
        findVCFSequenceBlack(this.board, Seq.empty)
      else
        findVCFSequenceWhite(this.board, Seq.empty)

  }

}
