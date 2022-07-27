package jrenju.solve

import jrenju.ParticleOps.particleOps
import jrenju.notation.{Flag, Pos, Renju}
import jrenju.{BitField, Board}
import utils.lang.Transform.IntTransform

import scala.collection.mutable

object LargeMoveGenerator extends MoveGenerator {

  def collectValidMoves(board: Board): Array[Int] = {
    var opponentHasOpenFour = false
    var opponentFiveAt = -1

    for (idx <- 0 until Renju.BOARD_SIZE) {
      val flag = board.boardField(idx)

      if (!Flag.isForbid(flag, board.nextColorFlag)) {
        val particle = board.structField(idx, board.nextColorFlag)
        val opponentParticle = board.structField(idx, board.colorFlag)

        if (particle.fiveTotal > 0)
          return Array(idx)

        if (opponentParticle.fiveTotal > 0)
          opponentFiveAt = idx

        if (opponentParticle.openFourTotal > 0)
          opponentHasOpenFour = true
      }
    }

    if (opponentFiveAt != -1)
      return Array(opponentFiveAt)

    if (opponentHasOpenFour) {
      val validMoves = mutable.ArrayBuilder.make[Int]

      for (idx <- 0 until Renju.BOARD_SIZE) {
        val flag = board.boardField(idx)

        if (!Flag.isForbid(flag, board.nextColorFlag)) {
          val particle = board.structField(idx, board.nextColorFlag)
          val opponentParticle = board.structField(idx, board.colorFlag)

          if (particle.fourTotal > 0 || particle.fiveTotal > 0)
            validMoves += idx

          if (opponentParticle.blockThreeTotal > 0)
            validMoves += idx
        }
      }

      return validMoves.result()
    }

    val moveField = BitField.empty(Renju.BOARD_WIDTH)

    for (idx <- 0 until Renju.BOARD_SIZE) {
      if (Flag.isExist(board.boardField(idx))) {
        val row = Pos.idxToRow(idx)
        val shift = Pos.idxToCol(idx) - 3

        // 1011 1010 0xba
        if (row > 2)
          moveField.applyMaskOr(row - 3, 0xba000000.integerShift(shift))
        // 0111 1100 0x7c
        if (row > 1)
          moveField.applyMaskOr(row - 2, 0x7c000000.integerShift(shift))
        // 1111 1110 0xfe
        if (row > 0)
          moveField.applyMaskOr(row - 1, 0xfe000000.integerShift(shift))
        // 1110 1110 0xee
        moveField.applyMaskOr(row, 0xee000000.integerShift(shift))
        // 1111 1110 0xfe
        if (row < Renju.BOARD_WIDTH - 1)
          moveField.applyMaskOr(row + 1, 0xfe000000.integerShift(shift))
        // 0111 1100 0x7c
        if (row < Renju.BOARD_WIDTH - 2)
          moveField.applyMaskOr(row + 2, 0x7c000000.integerShift(shift))
        // 1011 1010 0xba
        if (row < Renju.BOARD_WIDTH - 3)
          moveField.applyMaskOr(row + 3, 0xba000000.integerShift(shift))
      }
    }

    val validMoves = mutable.ArrayBuilder.make[Int]

    for (idx <- 0 until Renju.BOARD_SIZE)
      if (moveField(idx) && Flag.isEmpty(board.boardField(idx)) && !Flag.isForbid(board.boardField(idx), board.nextColorFlag))
        validMoves += idx

    validMoves.result()
  }

}
