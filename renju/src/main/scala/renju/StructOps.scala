package renju

import renju.notation._

import scala.collection.mutable
import scala.language.implicitConversions

final class StructOps(val b: Board) extends AnyVal {

  @inline private def getOffsetIdx(direction: Direction, initRow: Int, initCol: Int, offset: Int): Int = direction match {
    case Direction.X => Pos.rowColToIdx(initRow, initCol + offset)
    case Direction.Y => Pos.rowColToIdx(initRow + offset, initCol)
    case Direction.IncreaseUp => Pos.rowColToIdx(initRow + offset, initCol + offset)
    case Direction.DescentUp => Pos.rowColToIdx(initRow + offset, initCol - offset)
  }

  @inline private def getBoardFieldBounded(idx: Int): Byte =
    if (idx < 0 || idx >= Renju.BOARD_SIZE) Flag.WALL
    else b.field(idx)

  @inline private def processParticleBounded(idx: Int, process: Int => Boolean): Boolean =
    if (idx < 0 || idx >= Renju.BOARD_SIZE) false
    else process(idx)

  def collectOpen3Counters(direction: Direction, idx: Int, extract: Int => Struct, flag: Byte): Array[Int] = {
    val row = Pos.idxToRow(idx)
    val col = Pos.idxToCol(idx)

    val p2Pointer = this.getOffsetIdx(direction, row, col, -2)
    val p2Value = this.getBoardFieldBounded(p2Pointer)
    val p1Pointer = this.getOffsetIdx(direction, row, col, -1)
    val p1Value = this.getBoardFieldBounded(p1Pointer)
    val a1Pointer = this.getOffsetIdx(direction, row, col, 1)
    val a1Value = this.getBoardFieldBounded(a1Pointer)
    val a2Pointer = this.getOffsetIdx(direction, row, col, 2)
    val a2Value = this.getBoardFieldBounded(a2Pointer)

    // +0OO+
    if (a1Value == flag && a2Value == flag) {
      val builder = new mutable.ArrayBuilder.ofInt

      if (extract(p1Pointer).threeAt(direction))
        builder += p1Pointer
      val end = this.getOffsetIdx(direction, row, col, 3)
      if (extract(end).threeAt(direction))
        builder += end

      builder.result()
    }

    // +OO0+
    else if (p1Value == flag && p2Value == flag) {
      val builder = new mutable.ArrayBuilder.ofInt

      if (extract(a1Pointer).threeAt(direction))
        builder += a1Pointer
      val start = this.getOffsetIdx(direction, row, col, -3)
      if (extract(start).threeAt(direction))
        builder += start

      builder.result()
    }

    // O0+O
    else if (p1Value == flag && a2Value == flag)
      Array(a1Pointer)
    // O+0O
    else if (p2Value == flag && a1Value == flag)
      Array(p1Pointer)

    // -0O+O
    else if (
      Flag.isEmpty(p1Value)
        && a1Value == flag
        && Flag.isEmpty(p2Value)
    )
      Array(a2Pointer)

    // O+O0-
    else if (
      p1Value == flag
        && Flag.isEmpty(p2Value)
        && Flag.isEmpty(a1Value)
    )
      Array(p2Pointer)

    // +O0O+
    else if (p1Value == flag && a1Value == flag) {
      val builder = new mutable.ArrayBuilder.ofInt

      if (extract(a2Pointer).threeAt(direction))
        builder += a2Pointer
      if (extract(p2Pointer).threeAt(direction))
        builder += p2Pointer

      builder.result()
    }

    // OO+0
    else if (Flag.onlyStone(p1Value) == Flag.EMPTY && p2Value == flag)
      Array(p1Pointer)

    // 0+OO
    else if (Flag.onlyStone(a1Value) == Flag.EMPTY && a2Value == flag)
      Array(a1Pointer)

    else
      Array.empty
  }

  def collectClosed4Counter(direction: Direction, idx: Int, extract: Int => Struct): Int = {
    val row = Pos.idxToRow(idx)
    val col = Pos.idxToCol(idx)

    for (offset <- -5 to 5) {
      if (offset != 0) {
        val pointer = this.getOffsetIdx(direction, row, col, offset)
        if (this.processParticleBounded(pointer, extract(_).closedFourAt(direction)))
          return pointer
      }
    }

    -1
  }

  private def isNotPseudoThree(direction: Direction, idx: Int, from: Int): Boolean = {
    val counters = this.collectOpen3Counters(direction, idx, idx => b.structFieldBlack(idx), Flag.BLACK)
    for (counter <- counters) {
      val flag = b.field(counter)
      if (flag != Flag.FORBIDDEN_6 && flag != Flag.FORBIDDEN_44) {
        val struct = Struct(b.structFieldBlack(counter))
        if (struct.fourTotal == 0 && struct.fiveTotal == 0) {
          if (struct.threeTotal > 2) {
            if (this.isPseudoForbid(direction, counter, from))
              return true
          } else
            return true
        }
      }
    }

    b.structFieldBlack(idx) = Struct(b.structFieldBlack(idx)).merged(direction, 0x00000000).raw
    false
  }

  private def isPseudoForbid(idx: Int): Boolean = {
    var count = 0
    val struct = Struct(b.structFieldBlack(idx))
    for (direction <- Direction.values)
      if (struct.threeAt(direction) && this.isNotPseudoThree(direction, idx, idx))
        count += 1

    count < 2
  }

  private def isPseudoForbid(excludeDirection: Direction, idx: Int, from: Int): Boolean = {
    if (idx == from) return false

    var count = 0
    val struct = Struct(b.structFieldBlack(idx))
    for (direction <- Direction.values)
      if (direction != excludeDirection && struct.threeAt(direction) && this.isNotPseudoThree(direction, idx, from))
        count += 1

    count < 2
  }

  def collectTrapPoints(): (Array[Int], Array[Int]) = {
    val threeSideTraps = new mutable.ArrayBuilder.ofInt
    val fourSideTraps = new mutable.ArrayBuilder.ofInt

    for (idx <- 0 until Renju.BOARD_SIZE) {
      if (Flag.isForbid(b.field(idx))) {
        val struct = Struct(b.structFieldWhite(idx))

        for (direction <- Direction.values) {
          if (struct.threeAt(direction))
            threeSideTraps.addAll(this.collectOpen3Counters(direction, idx, idx => b.structFieldWhite(idx), Flag.WHITE))

          if (struct.closedFourAt(direction)) {
            val counter = this.collectClosed4Counter(direction, idx, idx => b.structFieldWhite(idx))
            if (counter != -1)
              fourSideTraps += counter
          }
        }
      }
    }

    (threeSideTraps.result(), fourSideTraps.result())
  }

  def calculateForbids(): Unit = {
    var hasDi3Forbid = false

    for (idx <- 0 until Renju.BOARD_SIZE) {
      val struct = Struct(b.structFieldBlack(idx))

      if (struct.fiveTotal > 0)
        b.field(idx) = Flag.EMPTY
      else if (b.field(idx) == Flag.FORBIDDEN_6)
        b.field(idx) = Flag.FORBIDDEN_6
      else if (struct.fourTotal > 1)
        b.field(idx) = Flag.FORBIDDEN_44
      else if (struct.threeTotal > 1) {
        b.field(idx) = Flag.FORBIDDEN_33
        hasDi3Forbid = true
      }
    }

    if (hasDi3Forbid)
      for (idx <- 0 until Renju.BOARD_SIZE)
        if (b.field(idx) == Flag.FORBIDDEN_33 && this.isPseudoForbid(idx))
          b.field(idx) = Flag.EMPTY
  }

}
