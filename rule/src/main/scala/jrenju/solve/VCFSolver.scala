package jrenju.solve

import jrenju.Board
import jrenju.BoardIO.BoardToText
import jrenju.PointOps.pointsOps
import jrenju.notation.{Flag, Renju}

//noinspection DuplicatedCode
object VCFSolver {

  private def findVCFSequenceBlack(memo: LRUMemo, maxDepth: Int, board: Board, parents: Seq[Int], coerce: Boolean): Seq[Int] = {
    if (parents.size > maxDepth || memo.probe(board).fold(false)(_ == 0)) return Seq.empty

    for (idx <- 0 until Renju.BOARD_SIZE) {
      val points = board.pointFieldBlack(idx)
      if (
        points.fourTotal == 1
          && !Flag.isForbid(board.boardField(idx))
          && (!coerce || board.pointFieldWhite(idx).fiveTotal == 1)
          && memo.probe(board, idx).fold(true)(_ != 0)
      ) {
        if (points.openFourTotal == 1) {
          memo.write(board, idx, Float.MaxValue)
          return parents.appended(idx)
        }

        val l1board = board.makeMove(idx, calculateForbid = false)

        val counter = l1board.pointFieldBlack.indexWhere(_.fiveTotal == 1)
        val counterPoint = l1board.pointFieldWhite(counter)

        val l2board = l1board.makeMove(counter)

        if (counterPoint.closedFourTotal < 2 && counterPoint.openFourTotal == 0) {
          if (
            (points.threeTotal == 1 && counterPoint.closedFourTotal == 0)
              || (l2board.pointFieldBlack.exists(_.openFourTotal == 1) && counterPoint.closedFourTotal == 0)
          ) {
            memo.write(board, idx, Float.MaxValue)
            return parents.appended(idx)
          }

          val solution = this.findVCFSequenceBlack(
            memo, maxDepth,
            l2board, parents.appended(idx).appended(counter),
            counterPoint.closedFourTotal == 1
          )

          if (solution.nonEmpty) {
            memo.write(l1board, Float.MaxValue)
            memo.write(l2board, Float.MinValue)
            return solution
          }
        }
      }

      memo.write(board, idx, 0)
    }

    Seq.empty
  }

  private def findVCFSequenceWhite(memo: LRUMemo, maxDepth: Int, board: Board, parents: Seq[Int], coerce: Boolean): Seq[Int] = {
    if (parents.size > maxDepth) return Seq.empty

    for (idx <- 0 until Renju.BOARD_SIZE) {
      val points = board.pointFieldWhite(idx)
      if (
        points.fourTotal == 1
          && (!coerce || board.pointFieldBlack(idx).fiveTotal == 1)
          && memo.probe(board, idx).fold(true)(_ != 0)
      ) {
        if (points.openFourTotal == 1) {
          memo.write(board, idx, Float.MaxValue)
          return parents.appended(idx)
        }

        val l1board = board.makeMove(idx, calculateForbid = false)

        val counter = l1board.pointFieldWhite.indexWhere(_.fiveTotal > 0)
        val counterPoint = l1board.pointFieldBlack(counter)

        val l2board = l1board.makeMove(counter)

        if (counterPoint.openFourTotal == 0 || Flag.isForbid(board.boardField(counter))) {
          if (
            (points.threeTotal > 1 || points.fourTotal > 1 && counterPoint.fourTotal == 0)
              || Flag.isForbid(board.boardField(counter))
              || (l2board.pointFieldWhite.exists(_.openFourTotal == 1) && counterPoint.closedFourTotal == 0)
          ) {
            memo.write(board, idx, Float.MaxValue)
            return parents.appended(idx)
          }

          val solution = this.findVCFSequenceWhite(
            memo, maxDepth,
            l2board, parents.appended(idx).appended(counter), counterPoint.closedFourTotal == 1
          )

          if (solution.nonEmpty) {
            memo.write(l1board, Float.MaxValue)
            memo.write(l2board, Float.MinValue)
            return solution
          }
        }
      }

      memo.write(board, idx, 0)
    }

    Seq.empty
  }

  implicit class VCFFinder(val b: Board) { self =>

    def isVCFRoot(memo: LRUMemo, maxDepth: Int): Boolean =
      self.findVCFSequence(memo, maxDepth).nonEmpty

    def findVCFSequence(memo: LRUMemo = LRUMemo.empty, maxDepth: Int = Int.MaxValue): Seq[Int] =
      if (self.b.isNextColorBlack)
        findVCFSequenceBlack(memo, maxDepth, self.b, Seq.empty, coerce = false)
      else
        findVCFSequenceWhite(memo, maxDepth, self.b, Seq.empty, coerce = false)

  }

}
