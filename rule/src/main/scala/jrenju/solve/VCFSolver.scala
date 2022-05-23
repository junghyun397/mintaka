package jrenju.solve

import jrenju.Board
import jrenju.notation.{Flag, Renju}

//noinspection DuplicatedCode
object VCFSolver {

  private def findVCFSequenceBlack(memo: LRUMemo, maxDepth: Int, board: Board, parents: Seq[Int], coerce: Boolean): Seq[Int] = {
    if (parents.size > maxDepth || memo.probe(board).fold(false)(_ == 0)) return Seq.empty

    for (idx <- 0 until Renju.BOARD_SIZE) {
      val point = board.pointsField(idx).black
      if (
        point.four == 1
          && !Flag.isForbid(board.boardField(idx))
          && (!coerce || board.pointsField(idx).white.fiveInRow == 1)
          && memo.probe(board, idx).fold(true)(_ != 0)
      ) {
        if (point.openFour == 1) {
          memo.write(board, idx, Float.MaxValue)
          return parents.appended(idx)
        }

        val l1board = board.makeMove(idx, calculateForbid = false)

        val counter = l1board.pointsField.indexWhere(_.black.fiveInRow == 1)
        val counterPoint = l1board.pointsField(counter).white

        val l2board = l1board
          .makeMove(counter)

        if (counterPoint.closedFour < 2 && counterPoint.openFour == 0) {
          if (
            (point.three == 1 && counterPoint.closedFour == 0)
              || (l2board.pointsField.exists(_.black.openFour == 1) && counterPoint.closedFour == 0)
          ) {
            memo.write(board, idx, Float.MaxValue)
            return parents.appended(idx)
          }

          val solution = this.findVCFSequenceBlack(
            memo, maxDepth,
            l2board, parents.appended(idx).appended(counter),
            counterPoint.closedFour == 1
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
      val points = board.pointsField(idx).white
      if (
        points.four == 1
          && (!coerce || board.pointsField(idx).black.fiveInRow == 1)
          && memo.probe(board, idx).fold(true)(_ != 0)
      ) {
        if (points.openFour == 1) {
          memo.write(board, idx, Float.MaxValue)
          return parents.appended(idx)
        }

        val l1board = board.makeMove(idx, calculateForbid = false)

        val counter = l1board.pointsField.indexWhere(_.white.fiveInRow > 0)
        val counterPoint = l1board.pointsField(counter).black

        val l2board = l1board
          .makeMove(counter)

        if (counterPoint.openFour == 0 || Flag.isForbid(board.boardField(counter))) {
          if (
            (points.three > 1 || points.four > 1 && counterPoint.four == 0)
              || Flag.isForbid(board.boardField(counter))
              || (l2board.pointsField.exists(_.white.openFour == 1) && counterPoint.closedFour == 0)
          ) {
            memo.write(board, idx, Float.MaxValue)
            return parents.appended(idx)
          }

          val solution = this.findVCFSequenceWhite(
            memo, maxDepth,
            l2board, parents.appended(idx).appended(counter), counterPoint.closedFour == 1
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

  implicit class VCFFinder(val board: Board) {

    def isVCFRoot(memo: LRUMemo, maxDepth: Int): Boolean =
      this.findVCFSequence(memo, maxDepth).nonEmpty

    def findVCFSequence(memo: LRUMemo = LRUMemo.empty, maxDepth: Int = Int.MaxValue): Seq[Int] =
      if (this.board.isNextColorBlack)
        findVCFSequenceBlack(memo, maxDepth, this.board, Seq.empty, coerce = false)
      else
        findVCFSequenceWhite(memo, maxDepth, this.board, Seq.empty, coerce = false)

  }

}
