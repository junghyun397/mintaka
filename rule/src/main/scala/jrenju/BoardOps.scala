package jrenju

import jrenju.notation.{Direction, Flag, Pos, Renju}

import scala.collection.mutable
import scala.language.implicitConversions

//noinspection DuplicatedCode
final class BoardOps(private val b: Board) extends AnyVal {

  private def collectStonesX(row: Int): Array[Byte] = {
    val stones = Array.ofDim[Byte](Renju.BOARD_WIDTH)
    for (idx <- 0 until Renju.BOARD_WIDTH)
      stones(idx) = Flag.onlyStone(b.boardField(Pos.rowColToIdx(row, idx)))
    stones
  }

  private def collectStonesY(col: Int): Array[Byte] = {
    val stones = Array.ofDim[Byte](Renju.BOARD_WIDTH)
    for (idx <- 0 until Renju.BOARD_WIDTH)
      stones(idx) = Flag.onlyStone(b.boardField(Pos.rowColToIdx(idx, col)))
    stones
  }

  private def collectStonesDEG45(size: Int, row: Int, col: Int): Array[Byte] = {
    val stones = Array.ofDim[Byte](size)
    for (idx <- 0 until size)
      stones(idx) = Flag.onlyStone(b.boardField(Pos.rowColToIdx(row + idx, col + idx)))
    stones
  }

  private def collectStonesDEG315(size: Int, row: Int, col: Int): Array[Byte] = {
    val stones = Array.ofDim[Byte](size)
    for (idx <- 0 until size)
      stones(idx) = Flag.onlyStone(b.boardField(Pos.rowColToIdx(row + idx, col - idx)))
    stones
  }

  private def composeStrips(pivot: Int): Array[L2Strip] = {
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

  private def composeGlobalStrips(): Array[L2Strip] = {
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

  private def applyParticle(direction: Int, idx: Int, points: PointsProvidePair, forbidMask: Byte): Unit = {
    if (b.pointsField(idx).isDifference(direction, points))
      b.pointsField(idx) = b.pointsField(idx).merged(direction, points)

    if (forbidMask != Flag.FREE)
      b.boardField(idx) = forbidMask
  }

  private def mergeParticle(direction: Int, idx: Int, points: PointsProvidePair, forbidMask: Byte): Unit = {
    if (b.pointsField(idx).isDifference(direction, points))
      b.pointsField(idx).merge(direction, points)

    if (forbidMask != Flag.FREE)
      b.boardField(idx) = forbidMask
  }

  private def integrateStrips(strips: Array[L2Strip], op: (Int, Int, PointsProvidePair, Byte) => Unit): Unit = {
    var winner = Option.empty[Byte]

    for (idx <- 0 until Renju.BOARD_LENGTH) {
      if (
        b.boardField(idx) == Flag.FORBIDDEN_33
          || (b.boardField(idx) == Flag.FORBIDDEN_44 && 1 > b.pointsField(idx).black.closedFour)
      )
        b.boardField(idx) = Flag.FREE
    }

    for (strip <- strips) {
      if (strip.winner != Flag.FREE) winner = Option(strip.winner)

      strip.direction match {
        case Direction.X => for (idx <- strip.pointsStrip.indices)
          op(
            Direction.X,
            strip.startIdx + idx,
            strip.pointsStrip(idx), strip.forbidMask(idx),
          )
        case Direction.Y => for (idx <- strip.pointsStrip.indices)
          op(
            Direction.Y,
            Pos.rowColToIdx(idx, Pos.idxToCol(strip.startIdx)),
            strip.pointsStrip(idx), strip.forbidMask(idx),
          )
        case Direction.DEG45 => for (idx <- strip.pointsStrip.indices)
          op(
            Direction.DEG45,
            Pos.rowColToIdx(Pos.idxToRow(strip.startIdx) + idx, Pos.idxToCol(strip.startIdx) + idx),
            strip.pointsStrip(idx), strip.forbidMask(idx),
          )
        case Direction.DEG315 => for (idx <- strip.pointsStrip.indices)
          op(
            Direction.DEG315,
            Pos.rowColToIdx(Pos.idxToRow(strip.startIdx) + idx, Pos.idxToCol(strip.startIdx) - idx),
            strip.pointsStrip(idx), strip.forbidMask(idx),
          )
      }
    }

    b.winner = winner
  }

  @inline def calculatePoints(change: Int): Board = {
    this.integrateStrips(this.composeStrips(change), this.applyParticle)
    b
  }

  @inline def calculateInjectedPoints(change: Int): Board = {
    this.integrateStrips(this.composeStrips(change), this.mergeParticle)
    b
  }

  @inline def calculateGlobalPoints(): Board = {
    this.integrateStrips(this.composeGlobalStrips(), this.applyParticle)
    b
  }

}
