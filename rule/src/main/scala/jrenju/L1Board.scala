//noinspection DuplicatedCode

package jrenju

import jrenju.notation._

import scala.collection.mutable

final class L1Board(
  boardField: Array[Byte],
  pointsField: Array[PointsPair],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening]
) extends Board(boardField, pointsField, moves, latestMove, opening) {

  def collectStonesX(row: Int): Array[Byte] = {
    val stones = Array.ofDim[Byte](Renju.BOARD_WIDTH)
    for (idx <- 0 until Renju.BOARD_WIDTH)
      stones(idx) = Flag.onlyStone(this.boardField(Pos.rowColToIdx(row, idx)))
    stones
  }

  def collectStonesY(col: Int): Array[Byte] = {
    val stones = Array.ofDim[Byte](Renju.BOARD_WIDTH)
    for (idx <- 0 until Renju.BOARD_WIDTH)
      stones(idx) = Flag.onlyStone(this.boardField(Pos.rowColToIdx(idx, col)))
    stones
  }

  def collectStonesDEG45(size: Int, row: Int, col: Int): Array[Byte] = {
    val stones = Array.ofDim[Byte](size)
    for (idx <- 0 until size)
      stones(idx) = Flag.onlyStone(this.boardField(Pos.rowColToIdx(row + idx, col + idx)))
    stones
  }

  def collectStonesDEG315(size: Int, row: Int, col: Int): Array[Byte] = {
    val stones = Array.ofDim[Byte](size)
    for (idx <- 0 until size)
      stones(idx) = Flag.onlyStone(this.boardField(Pos.rowColToIdx(row + idx, col - idx)))
    stones
  }

  def composeL2Strips(): Array[L2Strip] = this.composeL2Strips(this.latestMove)

  def composeL2Strips(pivot: Int): Array[L2Strip] = {
    val col = Pos.idxToCol(pivot)
    val row = Pos.idxToRow(pivot)

    val rCol = Renju.BOARD_MAX_IDX - col

    val builder = new mutable.ArrayBuilder.ofRef[L2Strip]()

    builder += new L1Strip(Direction.X, Pos.rowColToIdx(row, 0), this.collectStonesX(row))
      .calculateL2Strip()

    builder += new L1Strip(Direction.Y, Pos.rowColToIdx(0, col), this.collectStonesY(col))
      .calculateL2Strip()

    if (col - row < 0) { // TOP
      val y = row - col
      val size = Renju.BOARD_WIDTH - y
      if (size > 4)
        builder += new L1Strip(
          Direction.DEG45,
          Pos.rowColToIdx(y, 0),
          this.collectStonesDEG45(size, y, 0)
        )
          .calculateL2Strip()
    } else { // BOTTOM
      val x = col - row
      val size = Renju.BOARD_WIDTH - x
      if (size > 4)
        builder += new L1Strip(
          Direction.DEG45,
          Pos.rowColToIdx(0, x),
          this.collectStonesDEG45(size, 0, x)
        )
          .calculateL2Strip()
    }

    if (rCol - row < 0) { // TOP
      val y = row - rCol
      val size = Renju.BOARD_WIDTH - y
      if (size > 4)
        builder += new L1Strip(
          Direction.DEG315,
          Pos.rowColToIdx(y, Renju.BOARD_MAX_IDX),
          this.collectStonesDEG315(size, y, Renju.BOARD_MAX_IDX)
        )
          .calculateL2Strip()
    } else { // BOTTOM
      val x = rCol - row
      val size = Renju.BOARD_WIDTH - x
      if (size > 4)
        builder += new L1Strip(
          Direction.DEG315,
          Pos.rowColToIdx(0, col + row),
          this.collectStonesDEG315(size, 0, col + row)
        )
          .calculateL2Strip()
    }

    builder.result()
  }

  def composeGlobalL2Strips(): Array[L2Strip] = {
    val strips = Array.ofDim[L2Strip](Renju.BOARD_WIDTH * 6 - 18)

    for (idx <- 0 until Renju.BOARD_WIDTH) {
      strips(idx * 2) = new L1Strip(
        Direction.X,
        Pos.rowColToIdx(idx, 0),
        this.collectStonesX(idx)
      )
        .calculateL2Strip()

      strips(idx * 2 + 1) = new L1Strip(
        Direction.Y,
        Pos.rowColToIdx(0, idx),
        this.collectStonesY(idx)
      )
        .calculateL2Strip()
    }

    val offset45Bottom = Renju.BOARD_WIDTH * 2
    for (idx <- 0 until Renju.BOARD_WIDTH - 4) {
      strips(offset45Bottom + idx) = new L1Strip(
        Direction.DEG45,
        Pos.rowColToIdx(0, idx),
        this.collectStonesDEG45(Renju.BOARD_WIDTH - idx, 0, idx)
      )
        .calculateL2Strip()
    }

    val offset45Top = Renju.BOARD_WIDTH * 3 - 4
    for (idx <- 0 until Renju.BOARD_WIDTH - 5) {
      strips(offset45Top + idx) = new L1Strip(
        Direction.DEG45,
        Pos.rowColToIdx(idx + 1, 0),
        this.collectStonesDEG45(Renju.BOARD_MAX_IDX - idx, idx + 1, 0)
      )
        .calculateL2Strip()
    }

    val offset315Bottom = Renju.BOARD_WIDTH * 4 - 9
    for (idx <- 0 until Renju.BOARD_WIDTH - 4) {
      strips(offset315Bottom + idx) = new L1Strip(
        Direction.DEG315,
        Pos.rowColToIdx(0, Renju.BOARD_WIDTH - idx - 1),
        this.collectStonesDEG315(Renju.BOARD_WIDTH - idx, 0, Renju.BOARD_MAX_IDX - idx)
      )
        .calculateL2Strip()
    }

    val offset315Top = Renju.BOARD_WIDTH * 5 - 13
    for (idx <- 0 until Renju.BOARD_WIDTH - 5) {
      strips(offset315Top + idx) = new L1Strip(
        Direction.DEG315,
        Pos.rowColToIdx(idx + 1, Renju.BOARD_WIDTH - 1),
        this.collectStonesDEG315(Renju.BOARD_MAX_IDX - idx, idx + 1, Renju.BOARD_MAX_IDX)
      )
        .calculateL2Strip()
    }

    strips
  }

  private def mergeParticle(direction: Byte, idx: Int, points: PointsProvidePair, forbidMask: Byte): Unit = {
    if (this.pointsField(idx).isDifference(direction, points))
      this.pointsField(idx) = this.pointsField(idx).merged(direction, points)

    if (forbidMask != Flag.FREE)
      this.boardField(idx) = forbidMask
  }

  private def mergeL2Strips(strips: Array[L2Strip]): L2Board = {
    var winner = Flag.FREE

    for (idx <- 0 until Renju.BOARD_LENGTH) {
      if (
        this.boardField(idx) == Flag.FORBIDDEN_33
          || (this.boardField(idx) == Flag.FORBIDDEN_44 && 1 > this.pointsField(idx).black.closedFour)
      )
        this.boardField(idx) = Flag.FREE
    }

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

    new L2Board(this.boardField, pointsField, this.moves, this.latestMove, this.opening, winner)
  }

  def calculateL2Board(): L2Board =
    this.mergeL2Strips(this.composeL2Strips())

  def calculateGlobalL2Board(): L2Board =
    this.mergeL2Strips(this.composeGlobalL2Strips())

}
