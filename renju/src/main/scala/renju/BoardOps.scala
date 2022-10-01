//noinspection DuplicatedCode

package renju

import renju.notation._

import scala.collection.mutable
import scala.language.implicitConversions

final class BoardOps(val b: Board) extends AnyVal {

  private def collectStonesX(row: Int): Array[Byte] = {
    val stones = new Array[Byte](Renju.BOARD_WIDTH)
    for (idx <- 0 until Renju.BOARD_WIDTH)
      stones(idx) = Flag.onlyStone(b.field(Pos.rowColToIdx(row, idx)))
    stones
  }

  private def collectStonesY(col: Int): Array[Byte] = {
    val stones = new Array[Byte](Renju.BOARD_WIDTH)
    for (idx <- 0 until Renju.BOARD_WIDTH)
      stones(idx) = Flag.onlyStone(b.field(Pos.rowColToIdx(idx, col)))
    stones
  }

  private def collectStonesDEG45(size: Int, row: Int, col: Int): Array[Byte] = {
    val stones = new Array[Byte](size)
    for (idx <- 0 until size)
      stones(idx) = Flag.onlyStone(b.field(Pos.rowColToIdx(row + idx, col + idx)))
    stones
  }

  private def collectStonesDEG315(size: Int, row: Int, col: Int): Array[Byte] = {
    val stones = new Array[Byte](size)
    for (idx <- 0 until size)
      stones(idx) = Flag.onlyStone(b.field(Pos.rowColToIdx(row + idx, col - idx)))
    stones
  }

  def composeStrips(pivot: Int): Array[L1Strip] = {
    val col = Pos.idxToCol(pivot)
    val row = Pos.idxToRow(pivot)

    val rCol = Renju.BOARD_WIDTH_MAX_IDX - col

    val builder = new mutable.ArrayBuilder.ofRef[L1Strip]()

    builder += new L1Strip(Direction.X, Pos.rowColToIdx(row, 0), Renju.BOARD_WIDTH, this.collectStonesX(row))

    builder += new L1Strip(Direction.Y, Pos.rowColToIdx(0, col), Renju.BOARD_WIDTH, this.collectStonesY(col))

    if (col - row < 0) { // TOP
      val y = row - col
      val size = Renju.BOARD_WIDTH - y
      if (size > 4)
        builder += new L1Strip(
          Direction.IncreaseUp,
          Pos.rowColToIdx(y, 0),
          size,
          this.collectStonesDEG45(size, y, 0)
        )
    } else { // BOTTOM
      val x = col - row
      val size = Renju.BOARD_WIDTH - x
      if (size > 4)
        builder += new L1Strip(
          Direction.IncreaseUp,
          Pos.rowColToIdx(0, x),
          size,
          this.collectStonesDEG45(size, 0, x)
        )
    }

    if (rCol - row < 0) { // TOP
      val y = row - rCol
      val size = Renju.BOARD_WIDTH - y
      if (size > 4)
        builder += new L1Strip(
          Direction.DescentUp,
          Pos.rowColToIdx(y, Renju.BOARD_WIDTH_MAX_IDX),
          size,
          this.collectStonesDEG315(size, y, Renju.BOARD_WIDTH_MAX_IDX)
        )
    } else { // BOTTOM
      val x = rCol - row
      val size = Renju.BOARD_WIDTH - x
      if (size > 4)
        builder += new L1Strip(
          Direction.DescentUp,
          Pos.rowColToIdx(0, col + row),
          size,
          this.collectStonesDEG315(size, 0, col + row)
        )
    }

    builder.result()
  }

  def composeGlobalStrips(): Array[L1Strip] = {
    val strips = new Array[L1Strip](Renju.BOARD_WIDTH * 6 - 18)

    for (idx <- 0 until Renju.BOARD_WIDTH) {
      strips(idx * 2) = new L1Strip(
        Direction.X,
        Pos.rowColToIdx(idx, 0),
        Renju.BOARD_WIDTH,
        this.collectStonesX(idx)
      )

      strips(idx * 2 + 1) = new L1Strip(
        Direction.Y,
        Pos.rowColToIdx(0, idx),
        Renju.BOARD_WIDTH,
        this.collectStonesY(idx)
      )
    }

    val offset45Bottom = Renju.BOARD_WIDTH * 2
    for (idx <- 0 until Renju.BOARD_WIDTH - 4) {
      strips(offset45Bottom + idx) = new L1Strip(
        Direction.IncreaseUp,
        Pos.rowColToIdx(0, idx),
        Renju.BOARD_WIDTH - idx,
        this.collectStonesDEG45(Renju.BOARD_WIDTH - idx, 0, idx)
      )
    }

    val offset45Top = Renju.BOARD_WIDTH * 3 - 4
    for (idx <- 0 until Renju.BOARD_WIDTH - 5) {
      strips(offset45Top + idx) = new L1Strip(
        Direction.IncreaseUp,
        Pos.rowColToIdx(idx + 1, 0),
        Renju.BOARD_WIDTH_MAX_IDX - idx,
        this.collectStonesDEG45(Renju.BOARD_WIDTH_MAX_IDX - idx, idx + 1, 0)
      )
    }

    val offset315Bottom = Renju.BOARD_WIDTH * 4 - 9
    for (idx <- 0 until Renju.BOARD_WIDTH - 4) {
      strips(offset315Bottom + idx) = new L1Strip(
        Direction.DescentUp,
        Pos.rowColToIdx(0, Renju.BOARD_WIDTH - idx - 1),
        Renju.BOARD_WIDTH - idx,
        this.collectStonesDEG315(Renju.BOARD_WIDTH - idx, 0, Renju.BOARD_WIDTH_MAX_IDX - idx)
      )
    }

    val offset315Top = Renju.BOARD_WIDTH * 5 - 13
    for (idx <- 0 until Renju.BOARD_WIDTH - 5) {
      strips(offset315Top + idx) = new L1Strip(
        Direction.DescentUp,
        Pos.rowColToIdx(idx + 1, Renju.BOARD_WIDTH - 1),
        Renju.BOARD_WIDTH_MAX_IDX - idx,
        this.collectStonesDEG315(Renju.BOARD_WIDTH_MAX_IDX - idx, idx + 1, Renju.BOARD_WIDTH_MAX_IDX)
      )
    }

    strips
  }

  private def applyStruct(direction: Direction, idx: Int, blackStruct: Int, whiteStruct: Int, forbidMask: Byte): Unit = {
    b.structFieldBlack(idx) = Struct(b.structFieldBlack(idx)).merged(direction, blackStruct).raw
    b.structFieldWhite(idx) = Struct(b.structFieldWhite(idx)).merged(direction, whiteStruct).raw

    if (forbidMask != Flag.EMPTY)
      b.field(idx) = forbidMask
  }

  def integrateStrips(strips: Array[L2Strip]): Unit = {
    for (idx <- 0 until Renju.BOARD_SIZE) {
      if (
        b.field(idx) == Flag.FORBIDDEN_33
          || b.field(idx) == Flag.FORBIDDEN_44
      )
        b.field(idx) = Flag.EMPTY
    }

    for (strip <- strips) {
      strip.winner.foreach { color =>
        b.winner = Some(Result.FiveInRow(color))
      }

      strip.direction match {
        case Direction.X => for (idx <- 0 until strip.size)
          this.applyStruct(
            Direction.X,
            strip.startIdx + idx,
            strip.structStripBlack(idx), strip.structStripWhite(idx), strip.forbidMask(idx)
          )
        case Direction.Y => for (idx <- 0 until strip.size)
          this.applyStruct(
            Direction.Y,
            Pos.rowColToIdx(idx, Pos.idxToCol(strip.startIdx)),
            strip.structStripBlack(idx), strip.structStripWhite(idx), strip.forbidMask(idx)
          )
        case Direction.IncreaseUp => for (idx <- 0 until strip.size)
          this.applyStruct(
            Direction.IncreaseUp,
            Pos.rowColToIdx(Pos.idxToRow(strip.startIdx) + idx, Pos.idxToCol(strip.startIdx) + idx),
            strip.structStripBlack(idx), strip.structStripWhite(idx), strip.forbidMask(idx)
          )
        case Direction.DescentUp => for (idx <- 0 until strip.size)
          this.applyStruct(
            Direction.DescentUp,
            Pos.rowColToIdx(Pos.idxToRow(strip.startIdx) + idx, Pos.idxToCol(strip.startIdx) - idx),
            strip.structStripBlack(idx), strip.structStripWhite(idx), strip.forbidMask(idx)
          )
      }
    }
  }

}
