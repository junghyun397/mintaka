package jrenju.solve

import jrenju.Board
import jrenju.notation.{Flag, Renju}
import jrenju.protocol.{Solution, SolutionLeaf, SolutionNode}
import jrenju.ZobristHash.IncrementHash

object SolutionMapper {

  implicit class SolutionFinder(val m: LRUMemo) extends AnyVal {

    def findSolution(board: Board, threshold: Float): Option[Solution] = this.findSolution(threshold, board.moves, board.zobristKey)

    def findSolution(threshold: Float, moves: Int, hash: Long): Option[Solution] = {
      val color = Flag.colorFlag(moves + 1)
      val nextColor = Flag.colorFlag(moves)
      for (idx <- 0 until Renju.BOARD_SIZE) {
        val l1hash = hash.incrementHash(idx, color)
        if (m.probe(l1hash, color).fold(false)(_ == threshold)) {
          var counter = -1
          for (cIdx <- 0 until Renju.BOARD_SIZE) {
            if (m.probe(l1hash.incrementHash(cIdx, nextColor), nextColor).fold(false)(_ == -threshold))
              counter = cIdx
          }

          if (counter != -1) {

          }
        }
      }

      Option.empty
    }

  }

  implicit class SequenceToNode(sequence: Seq[Int]) {

    def toSolution: Solution = {
      val leaf = new SolutionLeaf(this.sequence.last)

      this.sequence
        .dropRight(1)
        .grouped(2)
        .foldRight[Solution](leaf) { (movePair, child) =>
          new SolutionNode(movePair.head, Map(movePair.last -> child))
        }
    }

  }

}
