package jrenju.solve

import jrenju.Board
import jrenju.ParticleOps.particleOps
import jrenju.notation.{Flag, Renju}

//noinspection DuplicatedCode
object VCFSolver {

  private def findVCFSequenceBlack(memo: LRUMemo, maxDepth: Int, board: Board, parents: Seq[Int], coerce: Boolean): Seq[Int] = {
    if (parents.size > maxDepth || memo.probe(board).fold(false)(_ == 0)) return Seq.empty

    for (idx <- 0 until Renju.BOARD_SIZE) {
      val particle = board.structFieldBlack(idx)
      if (
        particle.fourTotal == 1
          && !Flag.isForbid(board.field(idx))
          && (!coerce || board.structFieldWhite(idx).fiveTotal == 1)
          && memo.probe(board, idx).fold(true)(_ != 0)
      ) {
        if (particle.openFourTotal == 1) {
          memo.write(board, idx, Float.MaxValue)
          return parents.appended(idx)
        }

        val l1board = board.makeMove(idx, calculateForbid = false)

        val counter = l1board.structFieldBlack.indexWhere(_.fiveTotal > 0)
        val counterPoint = l1board.structFieldWhite(counter)

        val l2board = l1board.makeMove(counter)

        if (counterPoint.closedFourTotal < 2 && counterPoint.openFourTotal == 0) {
          if (
            (particle.threeTotal == 1 && counterPoint.closedFourTotal == 0)
              || (l2board.structFieldBlack.exists(_.openFourTotal == 1) && counterPoint.closedFourTotal == 0)
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
      val particle = board.structFieldWhite(idx)
      if (
        particle.fourTotal == 1
          && (!coerce || board.structFieldBlack(idx).fiveTotal == 1)
          && memo.probe(board, idx).fold(true)(_ != 0)
      ) {
        if (particle.openFourTotal == 1) {
          memo.write(board, idx, Float.MaxValue)
          return parents.appended(idx)
        }

        val l1board = board.makeMove(idx, calculateForbid = false)

        val counter = l1board.structFieldWhite.indexWhere(_.fiveTotal > 0)
        val counterPoint = l1board.structFieldBlack(counter)

        val l2board = l1board.makeMove(counter)

        if (counterPoint.openFourTotal == 0 || Flag.isForbid(board.field(counter))) {
          if (
            (particle.threeTotal > 1 || particle.fourTotal > 1 && counterPoint.fourTotal == 0)
              || Flag.isForbid(board.field(counter))
              || (l2board.structFieldWhite.exists(_.openFourTotal == 1) && counterPoint.closedFourTotal == 0)
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

  def findVCFSequence(board: Board, cache: LRUMemo = LRUMemo.empty, maxDepth: Int = Int.MaxValue): Seq[Int] =
    if (board.isNextColorBlack)
      findVCFSequenceBlack(cache, maxDepth, board, Seq.empty, coerce = false)
    else
      findVCFSequenceWhite(cache, maxDepth, board, Seq.empty, coerce = false)

}
