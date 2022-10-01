//noinspection DuplicatedCode

package engine.move

import renju.Struct.struct
import renju.notation.{Flag, Pos, Renju}
import renju.util.Transform.IntTransform
import renju.{BitField, Board, Struct}

import scala.collection.mutable

object LargeMoveGenerator extends MoveGenerator {

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

    val moveField = BitField.empty(Renju.BOARD_WIDTH)

    for (idx <- 0 until Renju.BOARD_SIZE) {
      if (Flag.isExist(board.field(idx))) {
        val row = Pos.idxToRow(idx)
        val shift = Pos.idxToCol(idx) - 3

        // 1011 1010 0xba
        if (row > 2)
          moveField.applyMaskOr(row - 3, 0xba000000 <<|>>> shift)
        // 0111 1100 0x7c
        if (row > 1)
          moveField.applyMaskOr(row - 2, 0x7c000000 <<|>>> shift)
        // 1111 1110 0xfe
        if (row > 0)
          moveField.applyMaskOr(row - 1, 0xfe000000 <<|>>> shift)
        // 1110 1110 0xee
        moveField.applyMaskOr(row, 0xee000000 <<|>>> shift)
        // 1111 1110 0xfe
        if (row < Renju.BOARD_WIDTH - 1)
          moveField.applyMaskOr(row + 1, 0xfe000000 <<|>>> shift)
        // 0111 1100 0x7c
        if (row < Renju.BOARD_WIDTH - 2)
          moveField.applyMaskOr(row + 2, 0x7c000000 <<|>>> shift)
        // 1011 1010 0xba
        if (row < Renju.BOARD_WIDTH - 3)
          moveField.applyMaskOr(row + 3, 0xba000000 <<|>>> shift)
      }
    }

    val validMoves = mutable.ArrayBuilder.make[Int]

    for (idx <- 0 until Renju.BOARD_SIZE)
      if (moveField(idx) && Flag.isEmpty(board.field(idx)) && !Flag.isForbid(board.field(idx), board.nextColorFlag.raw))
        validMoves += idx

    validMoves.result()
  }

}
