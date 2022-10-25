package engine.move

import renju.notation.Struct.struct
import renju.notation.{Flag, Pos, Renju, Struct}
import renju.util.Extensions.IntExtensions
import renju.{BitField, Board}

import scala.collection.mutable

object LargeMoveGenerator extends MoveGenerator {

  def collectValidMoves(board: Board): Array[Int] = {
    val moveField = BitField.empty(Renju.BOARD_WIDTH)

    for (idx <- 0 until Renju.BOARD_SIZE) {
      if (Flag.isExist(board.field(idx))) {
        val row = Pos.idxToRow(idx)
        val shift = Pos.idxToCol(idx) - 3

        // 1011 1010 0xba
        if (row > 2)
          moveField.applyMaskOr(row - 3, 0xba000000 shi shift)
        // 0111 1100 0x7c
        if (row > 1)
          moveField.applyMaskOr(row - 2, 0x7c000000 shi shift)
        // 1111 1110 0xfe
        if (row > 0)
          moveField.applyMaskOr(row - 1, 0xfe000000 shi shift)
        // 1110 1110 0xee
        moveField.applyMaskOr(row, 0xee000000 shi shift)
        // 1111 1110 0xfe
        if (row < Renju.BOARD_WIDTH - 1)
          moveField.applyMaskOr(row + 1, 0xfe000000 shi shift)
        // 0111 1100 0x7c
        if (row < Renju.BOARD_WIDTH - 2)
          moveField.applyMaskOr(row + 2, 0x7c000000 shi shift)
        // 1011 1010 0xba
        if (row < Renju.BOARD_WIDTH - 3)
          moveField.applyMaskOr(row + 3, 0xba000000 shi shift)
      }
    }

    val validMoves = mutable.ArrayBuilder.make[Int]

    for (idx <- 0 until Renju.BOARD_SIZE)
      if (moveField(idx) && Flag.isEmpty(board.field(idx)) && !Flag.isForbid(board.field(idx), board.nextColorFlag.raw))
        validMoves += idx

    validMoves.result()
  }

}
