//noinspection DuplicatedCode

package jrenju

import jrenju.notation.{Direction, Flag, Opening, Pos, Renju}

final class L1Board(
  boardField: Array[Byte],
  pointsField: Array[PointsPair],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening]
) extends Board(boardField, pointsField, moves, latestMove, opening) {

  def composeL2Strips(): Array[L2Strip] = this.composeL2Strips(this.latestMove)

  private def composeL2Strips(pivot: Int): Array[L2Strip] = {
    val col = Pos.idxToCol(pivot)
    val row = Pos.idxToRow(pivot)

    val rCol = Renju.BOARD_MAX_IDX - col

    Array(
      new L1Strip(
        Direction.X,
        Pos.rowColToIdx(row, 0),
        (for (idx <- 0 until Renju.BOARD_WIDTH)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(row, idx)))).toArray
      )
        .calculateL2Strip(),
      new L1Strip(
        Direction.Y,
        Pos.rowColToIdx(0, col),
        (for (idx <- 0 until Renju.BOARD_WIDTH)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(idx, col)))).toArray
      )
        .calculateL2Strip(),
      // 45 DEGREE STRIP
      if (col - row < 0) { // TOP
        val y = row - col
        val size = Renju.BOARD_WIDTH - y
        if (size > 4) new L1Strip(
          Direction.DEG45,
          Pos.rowColToIdx(y, 0),
          (for (idx <- 0 until size)
            yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(y + idx, idx)))).toArray
        )
          .calculateL2Strip()
        else null
      } else { // BOTTOM
        val x = col - row
        val size = Renju.BOARD_WIDTH - x
        if (size > 4) new L1Strip(
          Direction.DEG45,
          Pos.rowColToIdx(0, x),
          (for (idx <- 0 until size)
            yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(idx, x + idx)))).toArray
        )
          .calculateL2Strip()
        else null
      },
      // 315 DEGREE STRIP
      if (rCol - row < 0) { // TOP
        val y = row - rCol
        val size = Renju.BOARD_WIDTH - y
        if (size > 4) new L1Strip(
          Direction.DEG315,
          Pos.rowColToIdx(y, Renju.BOARD_MAX_IDX),
          (for (idx <- 0 until size)
            yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(y + idx, Renju.BOARD_MAX_IDX - idx)))).toArray
        )
          .calculateL2Strip()
        else null
      } else { // BOTTOM
        val x = rCol - row
        val size = Renju.BOARD_WIDTH - x
        if (size > 4) new L1Strip(
          Direction.DEG315,
          Pos.rowColToIdx(0, Renju.BOARD_MAX_IDX - rCol),
          (for (idx <- 0 until size)
            yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(idx, col + row - idx)))).toArray
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
        Pos.rowColToIdx(idx, 0),
        (for (col <- 0 until Renju.BOARD_WIDTH)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(idx, col)))).toArray
      )
        .calculateL2Strip()
      strips(idx * 2 + 1) = new L1Strip(
        Direction.Y,
        Pos.rowColToIdx(0, idx),
        (for (row <- 0 until Renju.BOARD_WIDTH)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(row, idx)))).toArray
      )
        .calculateL2Strip()
    }

    val offset45Bottom = Renju.BOARD_WIDTH * 2
    for (idx <- 0 until Renju.BOARD_WIDTH - 4) {
      strips(offset45Bottom + idx) = new L1Strip(
        Direction.DEG45,
        Pos.rowColToIdx(0, idx),
        (for (dIdx <- 0 until Renju.BOARD_WIDTH - idx)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(dIdx, idx + dIdx)))).toArray
      )
        .calculateL2Strip()
    }

    val offset45Top = Renju.BOARD_WIDTH * 3 - 4
    for (idx <- 0 until Renju.BOARD_WIDTH - 5) {
      strips(offset45Top + idx) = new L1Strip(
        Direction.DEG45,
        Pos.rowColToIdx(idx + 1, 0),
        (for (dIdx <- 0 until Renju.BOARD_WIDTH - idx - 1)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(idx + 1 + dIdx, dIdx)))).toArray
      )
        .calculateL2Strip()
    }

    val offset315Bottom = Renju.BOARD_WIDTH * 4 - 9
    for (idx <- 0 until Renju.BOARD_WIDTH - 4) {
      strips(offset315Bottom + idx) = new L1Strip(
        Direction.DEG315,
        Pos.rowColToIdx(0, Renju.BOARD_WIDTH - idx - 1),
        (for (dIdx <- 0 until Renju.BOARD_WIDTH - idx)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(dIdx, Renju.BOARD_WIDTH - idx - 1 - dIdx)))).toArray
      )
        .calculateL2Strip()
    }

    val offset315Top = Renju.BOARD_WIDTH * 5 - 13
    for (idx <- 0 until Renju.BOARD_WIDTH - 5) {
      strips(offset315Top + idx) = new L1Strip(
        Direction.DEG315,
        Pos.rowColToIdx(idx + 1, Renju.BOARD_WIDTH - 1),
        (for (dIdx <- 0 until Renju.BOARD_WIDTH - idx - 1)
          yield Flag.onlyStone(this.boardField(Pos.rowColToIdx(idx + 1 + dIdx, Renju.BOARD_WIDTH - 1 - dIdx)))).toArray
      )
        .calculateL2Strip()
    }

    strips
  }

  @inline private def mergeParticle(direction: Byte, idx: Int, points: PointsProvidePair, forbidMask: Byte): Unit = {
    this.pointsField(idx).black.merge(direction, points.black)
    this.pointsField(idx).white.merge(direction, points.white)

    if (forbidMask > Flag.FREE)
      this.boardField(idx) = forbidMask
  }

  private def mergeL2Strips(strips: Array[L2Strip]): L2Board = {
    var winner = Flag.FREE

    for (strip <- strips) {
      if (strip.winner != Flag.FREE) winner = strip.winner

      strip.direction match {
        case Direction.X => for (idx <- strip.pointsStrip.indices)
          this.mergeParticle(
            Direction.X,
            strip.startIdx + idx,
            strip.pointsStrip(idx), strip.forbidMask(idx),
          )
        case Direction.Y => for (idx <- strip.pointsStrip.indices)
          this.mergeParticle(
            Direction.Y,
            Pos.rowColToIdx(idx, Pos.idxToCol(strip.startIdx)),
            strip.pointsStrip(idx), strip.forbidMask(idx),
          )
        case Direction.DEG45 => for (idx <- strip.pointsStrip.indices)
          this.mergeParticle(
            Direction.DEG45,
            Pos.rowColToIdx(Pos.idxToRow(strip.startIdx) + idx, Pos.idxToCol(strip.startIdx) + idx),
            strip.pointsStrip(idx), strip.forbidMask(idx),
          )
        case Direction.DEG315 => for (idx <- strip.pointsStrip.indices)
          this.mergeParticle(
            Direction.DEG315,
            Pos.rowColToIdx(Pos.idxToRow(strip.startIdx) + idx, Pos.idxToCol(strip.startIdx) - idx),
            strip.pointsStrip(idx), strip.forbidMask(idx),
          )
      }
    }

    new L2Board(this.boardField, pointsField, this.moves, latestMove, this.opening, winner)
  }

  def calculateL2Board(): L2Board =
    this.mergeL2Strips(this.composeL2Strips())

  def calculateGlobalL2Board(): L2Board =
    this.mergeL2Strips(this.composeGlobalL2Strips())

}
