package jrenju.solve

import jrenju.Board
import jrenju.notation.{Flag, Renju}

//noinspection DuplicatedCode
object VCFSolver {

  def findVCFSequenceBlack(board: Board, parents: Seq[Int], coerce: Boolean, memo: LRUMemo, maxDepth: Int): Seq[Int] = {
    if (parents.size > maxDepth) return Seq.empty

    for (idx <- 0 until Renju.BOARD_LENGTH) {
      val points = board.pointsField(idx).black

      if (
        memo.probe(board, idx, Flag.BLACK).fold(true)(_ != 0)
          && points.closedFour == 1
          && !Flag.isForbid(board.boardField(idx))
          && (!coerce || board.pointsField(idx).white.fiveInRow == 1)
      ) {
        if (
          points.three == 1
            && board.pointsField(board.collectClosed4Companion(points.closed4.indexOf(1), idx, _.black)).white.four == 0
        ) {
          memo.write(board, idx, Flag.BLACK, Float.MaxValue)
          return parents.appended(idx)
        }

        val companion = board.collectClosed4Companion(points.closed4.indexOf(1), idx, _.black)

        val l1board = board
          .makeMove(idx, calculateForbid = false)
        val l2board = l1board
          .makeMove(companion)

        val companionPoint = l1board.pointsField(companion).white
        val white4 = companionPoint.four
        if (white4 < 2 && companionPoint.closedFour - white4 == 0) {
          val solution = this.findVCFSequenceBlack(l2board, parents.appended(idx).appended(companion), white4 == 1, memo, maxDepth)

          if (solution.nonEmpty) {
            memo.write(l1board, Float.MaxValue)
            memo.write(l2board, Float.MinValue)
            return solution
          }
        }
      }

      memo.write(board, idx, Flag.BLACK, 0)
    }

    Seq.empty
  }

  def findVCFSequenceWhite(board: Board, parents: Seq[Int], coerce: Boolean, memo: LRUMemo, maxDepth: Int): Seq[Int] = {
    if (parents.size > maxDepth) return Seq.empty

    for (idx <- 0 until Renju.BOARD_LENGTH) {
      val points = board.pointsField(idx).white

      if (
        memo.probe(board, idx, Flag.WHITE).fold(true)(_ != 0)
          && points.closedFour == 1
          && (!coerce || board.pointsField(idx).black.fiveInRow == 1)
      ) {
        if (
          points.three > 1 || points.four > 1
            && board.pointsField(board.collectClosed4Companion(points.closed4.indexOf(1), idx, _.white)).black.four == 0
        ) {
          memo.write(board, idx, Flag.WHITE, Float.MaxValue)
          return parents.appended(idx)
        }

        val companion = board.collectClosed4Companion(points.closed4.indexOf(1), idx, _.white)

        if (Flag.isForbid(board.boardField(companion))) {
          memo.write(board, idx, Flag.WHITE, Float.MaxValue)
          return parents.appended(idx)
        }

        val l1board = board
          .makeMove(idx, calculateForbid = false)
        val l2board = l1board
          .makeMove(companion)

        val companionPoint = l1board.pointsField(companion).black
        val black4 = companionPoint.four
        if (black4 < 2 && companionPoint.closedFour - black4 == 0) {
          val solution = this.findVCFSequenceWhite(l2board, parents.appended(idx).appended(companion), black4 == 1, memo, maxDepth)

          if (solution.nonEmpty) {
            memo.write(l1board, Float.MaxValue)
            memo.write(l2board, Float.MinValue)
            return solution
          }
        }
      }

      memo.write(board, idx, Flag.WHITE, 0)
    }

    Seq.empty
  }


  implicit class VCFFinder(val board: Board) {

    def findVCFSequence(memo: LRUMemo = new LRUMemo(), maxDepth: Int = Int.MaxValue): Seq[Int] =
      if (this.board.moves % 2 == 0)
        findVCFSequenceBlack(this.board, Seq.empty, coerce = false, memo, maxDepth)
      else
        findVCFSequenceWhite(this.board, Seq.empty, coerce = false, memo, maxDepth)

  }

}
