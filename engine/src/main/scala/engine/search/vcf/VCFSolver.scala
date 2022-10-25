//noinspection DuplicatedCode

package engine.search.vcf

import engine.cache.TranspositionTable
import renju.Board
import renju.notation.Struct.struct
import renju.notation.{Flag, Renju, Struct}

object VCFSolver {

  private def findVCFSequenceBlack(tt: TranspositionTable, maxDepth: Int, board: Board, parents: Seq[Int], treat: Boolean): Seq[Int] = {
    if (parents.size > maxDepth || tt.find(board.hashKey).vcPass) return Seq.empty

    for (idx <- 0 until Renju.BOARD_SIZE) {
      val struct = board.structFieldBlack(idx)
      if (
        struct.fourTotal == 1
          && !Flag.isForbid(board.field(idx))
          && (!treat || board.structFieldWhite(idx).fiveTotal == 1)
          && tt.find(board.hashKey.move(idx, board.nextColorFlag.raw)).isEmpty
      ) {
        if (struct.openFourTotal == 1) {
          tt.write(board.hashKey, eval = Int.MaxValue, bestMove = idx, depth = board.moves)
          return parents.appended(idx)
        }

        val l1board = board.makeMove(idx, calculateForbid = false)

        val counter = l1board.structFieldBlack.indexWhere(_.fiveTotal > 0)
        val counterStruct = l1board.structFieldWhite(counter)

        val l2board = l1board.makeMove(counter)

        if (counterStruct.closedFourTotal < 2 && counterStruct.openFourTotal == 0) {
          if (
            (struct.threeTotal == 1 && counterStruct.closedFourTotal == 0)
              || (l2board.structFieldBlack.exists(_.openFourTotal == 1) && counterStruct.closedFourTotal == 0)
          ) {
            tt.write(board.hashKey, eval = Int.MaxValue, bestMove = idx, depth = board.moves)
            return parents.appended(idx)
          }

          val solution = this.findVCFSequenceBlack(
            tt, maxDepth,
            l2board, parents.appended(idx).appended(counter),
            counterStruct.closedFourTotal == 1
          )

          if (solution.nonEmpty) {
            tt.write(l1board.hashKey, eval = Int.MinValue, depth = l1board.moves + 1)
            return solution
          }
        }
      }

      tt.write(board.hashKey.move(idx, board.nextColorFlag.raw), vcPass = true, depth = board.moves + 1)
    }

    Seq.empty
  }

  private def findVCFSequenceWhite(tt: TranspositionTable, maxDepth: Int, board: Board, parents: Seq[Int], treat: Boolean): Seq[Int] = {
    if (parents.size > maxDepth || tt.find(board.hashKey).vcPass) return Seq.empty

    for (idx <- 0 until Renju.BOARD_SIZE) {
      val struct = board.structFieldWhite(idx)
      if (
        struct.fourTotal == 1
          && (!treat || board.structFieldBlack(idx).fiveTotal == 1)
          && tt.find(board.hashKey.move(idx, board.nextColorFlag.raw)).isEmpty
      ) {
        if (struct.openFourTotal == 1) {
          tt.write(board.hashKey, eval = Int.MaxValue, bestMove = idx, depth = board.moves)
          return parents.appended(idx)
        }

        val l1board = board.makeMove(idx, calculateForbid = false)

        val counter = l1board.structFieldWhite.indexWhere(_.fiveTotal > 0)
        val counterStruct = l1board.structFieldBlack(counter)

        val l2board = l1board.makeMove(counter)

        if (counterStruct.openFourTotal == 0 || Flag.isForbid(board.field(counter))) {
          if (
            (struct.threeTotal > 1 || struct.fourTotal > 1 && counterStruct.fourTotal == 0)
              || Flag.isForbid(board.field(counter))
              || (l2board.structFieldWhite.exists(_.openFourTotal == 1) && counterStruct.closedFourTotal == 0)
          ) {
            tt.write(board.hashKey, eval = Int.MaxValue, bestMove = idx, depth = board.moves)
            return parents.appended(idx)
          }

          val solution = this.findVCFSequenceWhite(
            tt, maxDepth,
            l2board, parents.appended(idx).appended(counter), counterStruct.closedFourTotal == 1
          )

          if (solution.nonEmpty) {
            tt.write(l1board.hashKey, eval = Int.MinValue, depth = l2board.moves + 1)
            return solution
          }
        }
      }

      tt.write(board.hashKey.move(idx, board.nextColorFlag.raw), vcPass = true, depth = board.moves + 1)
    }

    Seq.empty
  }

  def findVCFSequence(board: Board): Seq[Int] = this.findVCFSequence(board, TranspositionTable.empty, Int.MaxValue)

  def findVCFSequence(board: Board, tt: TranspositionTable, maxDepth: Int): Seq[Int] =
    if (board.isNextColorBlack)
      findVCFSequenceBlack(tt, maxDepth, board, Seq.empty, treat = false)
    else
      findVCFSequenceWhite(tt, maxDepth, board, Seq.empty, treat = false)

}
