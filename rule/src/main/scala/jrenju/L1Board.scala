//noinspection DuplicatedCode

package jrenju

import jrenju.notation.{Flag, Opening, Pos}
import jrenju.rule.Renju

final class L1Board(
  boardField: Array[Byte],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening]
) extends Board(boardField, moves, latestMove, opening) {

  def composeL2Strips(): Array[L2Strip] = this.composeL2Strips(this.latestMove)

  private def composeL2Strips(pivot: Int): Array[L2Strip] = {
    val row = Pos.idxToRow(pivot)
    val col = Pos.idxToCol(pivot)

    val rRow = Renju.BOARD_MAX_IDX - row

    Array(
      new L1Strip(
        Direction.X,
        Pos.rowColToIdx(0, col),
        (for (idx <- 0 until Renju.BOARD_WIDTH)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(idx, col)))).toArray
      )
        .calculateL2Strip(),
      new L1Strip(
        Direction.Y,
        Pos.rowColToIdx(row, 0),
        (for (idx <- 0 until Renju.BOARD_WIDTH)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(row, idx)))).toArray
      )
        .calculateL2Strip(),
      // 45 DEGREE STRIP
      if (row - col < 0) { // TOP
        val y = col - row
        val size = Renju.BOARD_WIDTH - y
        if (size > 4) new L1Strip(
          Direction.DEG45,
          Pos.rowColToIdx(0, y),
          (for (idx <- 0 until size)
            yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(idx, y + idx)))).toArray
        )
          .calculateL2Strip()
        else null
      } else { // BOTTOM
        val x = row - col
        val size = Renju.BOARD_WIDTH - x
        if (size > 4) new L1Strip(
          Direction.DEG45,
          Pos.rowColToIdx(x, 0),
          (for (idx <- 0 until size)
            yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(x + idx, idx)))).toArray
        )
          .calculateL2Strip()
        else null
      },
      // 315 DEGREE STRIP
      if (rRow - col < 0) { // TOP
        val y = col - rRow
        val size = Renju.BOARD_WIDTH - y
        if (size > 4) new L1Strip(
          Direction.DEG315,
          Pos.rowColToIdx(Renju.BOARD_MAX_IDX, y),
          (for (idx <- 0 until size)
            yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(Renju.BOARD_MAX_IDX - idx, y + idx)))).toArray
        )
          .calculateL2Strip()
        else null
      } else { // BOTTOM
        val x = rRow - col
        val size = Renju.BOARD_WIDTH - x
        if (size > 4) new L1Strip(
          Direction.DEG315,
          Pos.rowColToIdx(Renju.BOARD_MAX_IDX - rRow, 0),
          (for (idx <- 0 until size)
            yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(row + col - idx, idx)))).toArray
        )
          .calculateL2Strip()
        else null
      }
    )
      .filterNot(_ == null)
  }

  def composeGlobalL2Strips(): Array[L2Strip] = {
    val strips = Array.fill[L2Strip](Renju.BOARD_WIDTH * 6 - 18)(null)

    for (idx <- 0 until Renju.BOARD_WIDTH) {
      strips(idx * 2) = new L1Strip(
        Direction.X,
        Pos.rowColToIdx(0, idx),
        (for (row <- 0 until Renju.BOARD_WIDTH)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(row, idx)))).toArray
      )
        .calculateL2Strip()
      strips(idx * 2 + 1) = new L1Strip(
        Direction.Y,
        Pos.rowColToIdx(0, idx),
        (for (col <- 0 until Renju.BOARD_WIDTH)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(idx, col)))).toArray
      )
        .calculateL2Strip()
    }

    val offset45Bottom = Renju.BOARD_WIDTH * 2
    for (idx <- 0 until Renju.BOARD_WIDTH - 4) {
      strips(offset45Bottom + idx) = new L1Strip(
        Direction.DEG45,
        Pos.rowColToIdx(idx, 0),
        (for (dIdx <- 0 until Renju.BOARD_WIDTH - idx)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(idx + dIdx, dIdx)))).toArray
      )
        .calculateL2Strip()
    }

    val offset45Top = Renju.BOARD_WIDTH * 3 - 4
    for (idx <- 0 until Renju.BOARD_WIDTH - 5) {
      strips(offset45Top + idx) = new L1Strip(
        Direction.DEG45,
        Pos.rowColToIdx(0, idx + 1),
        (for (dIdx <- 0 until Renju.BOARD_WIDTH - idx - 1)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(dIdx, idx + 1 + dIdx)))).toArray
      )
        .calculateL2Strip()
    }

    val offset315Bottom = Renju.BOARD_WIDTH * 4 - 9
    for (idx <- 0 until Renju.BOARD_WIDTH - 4) {
      strips(offset315Bottom + idx) = new L1Strip(
        Direction.DEG315,
        Pos.rowColToIdx(Renju.BOARD_WIDTH - idx - 1, 0),
        (for (dIdx <- 0 until Renju.BOARD_WIDTH - idx)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(Renju.BOARD_WIDTH - idx - 1 - dIdx, dIdx)))).toArray
      )
        .calculateL2Strip()
    }

    val offset315Top = Renju.BOARD_WIDTH * 5 - 13
    for (idx <- 0 until Renju.BOARD_WIDTH - 5) {
      strips(offset315Top + idx) = new L1Strip(
        Direction.DEG315,
        Pos.rowColToIdx(0, idx + 1),
        (for (dIdx <- 0 until Renju.BOARD_WIDTH - idx - 1)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(Renju.BOARD_WIDTH - 1 - dIdx, idx + 1 + dIdx)))).toArray
      )
        .calculateL2Strip()
    }

    strips
  }

  @inline private def mergeAttackPoint(target: AttackPoints, mod: AttackPoints): Unit = {
    target.five += mod.five
    target.open4 += mod.open4
    target.closed4 += mod.closed4
    target.open3 += mod.open3
  }

  @inline private def mergeParticle(attackField: Array[(AttackPoints, AttackPoints)], idx: Int, attackPoints: (AttackPoints, AttackPoints), forbiddenMask: Byte): Unit = {
    this.mergeAttackPoint(attackField(idx)._1, attackPoints._1)
    this.mergeAttackPoint(attackField(idx)._2, attackPoints._2)
    if (forbiddenMask > Flag.FREE) this.boardField(idx) = forbiddenMask
  }

  private def assembleL2Strips(strips: Array[L2Strip]): L2Board = {
    val attackField = Array.fill(Renju.BOARD_LENGTH)((new AttackPoints(), new AttackPoints()))

    var winner = Flag.FREE

    for (strip <- strips) {
      if (strip.winner != Flag.FREE) winner = strip.winner

      strip.direction match {
        case Direction.X => for (idx <- strip.attackStrip.indices)
          this.mergeParticle(
            attackField,
            strip.startIdx + idx,
            strip.attackStrip(idx), strip.forbiddenMask(idx),
          )
        case Direction.Y => for (idx <- strip.attackStrip.indices)
          this.mergeParticle(
            attackField,
            Pos.rowColToIdx(Pos.idxToRow(strip.startIdx), idx),
            strip.attackStrip(idx), strip.forbiddenMask(idx),
          )
        case Direction.DEG45 => for (idx <- strip.attackStrip.indices)
          this.mergeParticle(
            attackField,
            Pos.rowColToIdx(Pos.idxToRow(strip.startIdx) + idx, Pos.idxToCol(strip.startIdx) + idx),
            strip.attackStrip(idx), strip.forbiddenMask(idx),
          )
        case Direction.DEG315 => for (idx <- strip.attackStrip.indices)
          this.mergeParticle(
            attackField,
            Pos.rowColToIdx(Pos.idxToRow(strip.startIdx) - idx, Pos.idxToCol(strip.startIdx) + idx),
            strip.attackStrip(idx), strip.forbiddenMask(idx),
          )
      }
    }

    new L2Board(this.boardField, attackField, this.moves, latestMove, this.opening, winner)
  }

  def calculateL2Board(): L2Board =
    this.assembleL2Strips(this.composeL2Strips())

  def calculateGlobalL2Board(): L2Board =
    this.assembleL2Strips(this.composeGlobalL2Strips())

}
