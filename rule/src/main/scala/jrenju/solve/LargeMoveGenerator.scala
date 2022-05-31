package jrenju.solve

import jrenju.ParticleOps.particleOps
import jrenju.{BitField, Board}
import jrenju.notation.{Flag, Pos, Renju}

import scala.collection.mutable

object LargeMoveGenerator extends MoveGenerator {

  def collectValidMoves(board: Board): Array[Int] = {
    for (idx <- 0 until Renju.BOARD_SIZE) {
      val flag = board.boardField(idx)

      if (!Flag.isForbid(flag, board.nextColorFlag)) {
        val particle = board.structField(idx, board.nextColorFlag)

        if (particle.fiveTotal > 0)
          return Array(idx)
      }
    }

    val moveField = BitField.empty(Renju.BOARD_WIDTH)

    for (idx <- 0 until Renju.BOARD_SIZE) {
      val row = Pos.idxToRow(idx)
      val shift = Pos.idxToCol(idx) - 3

      if (Flag.isExist(board.boardField(idx))) {
        // 1011 1010 0xba
        if (row > 3)
          moveField.applyMaskOr(row - 3, shift, 0xba000000)
        // 0111 1100 0x7c
        if (row > 2)
          moveField.applyMaskOr(row - 2, shift, 0x7c000000)
        // 1111 1110 0xfe
        if (row > 1)
          moveField.applyMaskOr(row - 1, shift, 0xfe000000)
        // 1110 1110 0xee
        moveField.applyMaskOr(row, shift, 0xee000000)
        // 1111 1110 0xfe
        if (row < Renju.BOARD_WIDTH - 1)
          moveField.applyMaskOr(row + 1, shift, 0xfe000000)
        // 0111 1100 0x7c
        if (row < Renju.BOARD_WIDTH - 2)
          moveField.applyMaskOr(row + 2, shift, 0x7c000000)
        // 1011 1010 0xba
        if (row < Renju.BOARD_WIDTH - 3)
          moveField.applyMaskOr(row + 3, shift, 0xba000000)
      }
    }

    val validMoves = mutable.ArrayBuilder.make[Int]

    for (idx <- 0 until Renju.BOARD_SIZE)
      if (moveField(idx) && !Flag.isForbid(board.boardField(idx), board.nextColorFlag))
        validMoves += idx

    validMoves.result()
  }

}
