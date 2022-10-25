package engine.move

import renju.Board
import renju.notation.{Flag, Renju}

import scala.collection.mutable

object ThreatMoveGenerator extends MoveGenerator {

  def collectValidMoves(board: Board): Array[Int] = {
    var opponentHasOpenFour = false
    var opponentFiveAt = -1
    
    for (idx <- 0 until Renju.BOARD_SIZE) {
      val flag = Flag(board.field(idx))

      if (!flag.isForbid(board.nextColorFlag.raw)) {
        val struct = board.structField(idx, board.nextColorFlag.raw)
        val opponentStruct = board.structField(idx, board.colorFlag.raw)

        if (struct.fiveTotal > 0)
          return Array(idx)

        if (opponentStruct.fiveTotal > 0)
          opponentFiveAt = idx

        if (opponentStruct.openFourTotal > 0)
          opponentHasOpenFour = true
      }
    }

    if (opponentFiveAt != -1)
      return Array(opponentFiveAt)

    if (opponentHasOpenFour) {
      val validMoves = mutable.ArrayBuilder.make[Int]

      for (idx <- 0 until Renju.BOARD_SIZE) {
        val flag = board.field(idx)

        if (!board.nextColorFlag.isForbid(flag)) {
          val struct = board.structField(idx, board.nextColorFlag.raw)
          val opponentStruct = board.structField(idx, board.colorFlag.raw)

          if (struct.fourTotal > 0 || struct.fiveTotal > 0)
            validMoves += idx

          if (opponentStruct.blockThreeTotal > 0)
            validMoves += idx
        }
      }

      return validMoves.result()
    }
    
    Array.empty
  }

}
