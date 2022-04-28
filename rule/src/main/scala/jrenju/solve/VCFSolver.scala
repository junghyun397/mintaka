package jrenju.solve

import jrenju.Board
import jrenju.notation.{Flag, Renju}

object VCFSolver {

  def hasVCFBlack(board: Board, memo: ZobristHash.Memo): Boolean = {
    for (idx <- 0 until Renju.BOARD_LENGTH) {
      val points = board.pointsField(idx).black

      if (points.closedFour == 1) {
        if (points.three == 1)
          return true

        val companion = board.recoverClosed4Companion(points.closed4.indexOf(1), idx, _.black)

        if (board.pointsField(companion).white.four < 1) {
          board
            .injectMove(idx)
            .injectMove(companion)
            .calculateForbids(false)

          val hasVCF = this.hasVCFBlack(board, memo)

          board
            .removeMove(companion)
            .removeMove(idx)

          if (hasVCF)
            return true
        }
      }
    }

    false
  }

  def hasVCFWhite(board: Board, memo: ZobristHash.Memo): Boolean = {
    for (idx <- 0 until Renju.BOARD_LENGTH) {
      val points = board.pointsField(idx).white
      if (points.closed4.sum == 1) {
        if (points.three > 1 || points.four > 1)
          return true

        val companion = board.recoverClosed4Companion(points.closed4.indexOf(1), idx, _.white)

        if (Flag.isForbid(board.boardField(companion)))
          return true
        else if (board.pointsField(companion).black.four < 1) {
          board
            .injectMove(idx)
            .injectMove(companion)
            .calculateForbids(false)

          val hasVCF = this.hasVCFWhite(board, memo)

          board
            .removeMove(companion)
            .removeMove(idx)

          if (hasVCF)
            return true
        }
      }
    }

    false
  }

  def findVCFSequenceBlack(board: Board, parents: Seq[Int]): Seq[Int] = {
    for (idx <- 0 until Renju.BOARD_LENGTH) {
      val points = board.pointsField(idx).black

      if (points.closedFour == 1) {
        if (points.three == 1)
          return parents.appended(idx)

        val companion = board.recoverClosed4Companion(points.closed4.indexOf(1), idx, _.black)

        if (board.pointsField(companion).white.four < 1) {
          board
            .injectMove(idx)
            .injectMove(companion)
            .calculateForbids(false)

          val solution = this.findVCFSequenceBlack(board, parents.appended(idx).appended(companion))

          board.removeMove(companion)
          board.removeMove(idx)

          if (solution.nonEmpty)
            return solution
        }
      }
    }

    Seq.empty
  }

  def findVCFSequenceWhite(board: Board, parents: Seq[Int]): Seq[Int] = {
    for (idx <- 0 until Renju.BOARD_LENGTH) {
      val points = board.pointsField(idx).white
      if (points.closed4.sum == 1) {
        if (points.three > 1 || points.four > 1)
          return parents.appended(idx)

        val companion = board.recoverClosed4Companion(points.closed4.indexOf(1), idx, _.white)

        if (Flag.isForbid(board.boardField(companion)))
          return parents.appended(idx)
        else if (board.pointsField(companion).black.four < 1) {
          board
            .injectMove(idx)
            .injectMove(companion)
            .calculateForbids(false)

          val solution = this.findVCFSequenceWhite(board, parents.appended(idx).appended(companion))

          board
            .removeMove(companion)
            .removeMove(idx)

          if (solution.nonEmpty)
            return solution
        }
      }
    }

    Seq.empty
  }


  implicit class VCFFinder(val board: Board) {

    def hasVCFPoint(idx: Int): Boolean = ???

    def findVCFSequence(): Seq[Int] =
      if (this.board.moves % 2 == 0)
        findVCFSequenceBlack(this.board, Seq.empty)
      else
        findVCFSequenceWhite(this.board, Seq.empty)

  }

}
