package jrenju

import jrenju.PointOps.pointsOps
import jrenju.notation.{Direction, Flag, Pos, Renju}

import scala.collection.mutable
import scala.language.implicitConversions

//noinspection DuplicatedCode
final class StructOps(private val b: Board) extends AnyVal {

  @inline private def getOffsetIdx(direction: Int, initRow: Int, initCol: Int, offset: Int): Int = direction match {
    case Direction.X => Pos.rowColToIdx(initRow, initCol + offset)
    case Direction.Y => Pos.rowColToIdx(initRow + offset, initCol)
    case Direction.DEG45 => Pos.rowColToIdx(initRow + offset, initCol + offset)
    case Direction.DEG315 => Pos.rowColToIdx(initRow + offset, initCol - offset)
  }

  @inline private def getBoardFieldBounded(idx: Int): Byte =
    if (idx < 0 || idx >= Renju.BOARD_SIZE) Flag.WALL
    else b.boardField(idx)

  @inline private def getPointsBounded(idx: Int, process: Int => Boolean): Boolean =
    if (idx < 0 || idx >= Renju.BOARD_SIZE) false
    else process(idx)

  def collectOpen3Counters(direction: Int, idx: Int, extract: Int => Int, flag: Byte): Array[Int] = {
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
      Flag.onlyStone(p1Value) == Flag.FREE
        && a1Value == flag
        && Flag.onlyStone(p2Value) == Flag.FREE
    )
      Array(a2Pointer)

    // O+O0-
    else if (
      p1Value == flag
        && Flag.onlyStone(p2Value) == Flag.FREE
        && Flag.onlyStone(a1Value) == Flag.FREE
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
    else if (Flag.onlyStone(p1Value) == Flag.FREE && p2Value == flag)
      Array(p1Pointer)

    // 0+OO
    else if (Flag.onlyStone(a1Value) == Flag.FREE && a2Value == flag)
      Array(a1Pointer)

    else
      Array.empty
  }

  def collectClosed4Counter(direction: Int, idx: Int, extract: Int => Int): Int = {
    val row = Pos.idxToRow(idx)
    val col = Pos.idxToCol(idx)

    for (offset <- -5 to 5) {
      if (offset != 0) {
        val pointer = this.getOffsetIdx(direction, row, col, offset)
        if (this.getPointsBounded(pointer, extract(_).closedFourAt(direction)))
          return pointer
      }
    }

    -1
  }

  private def isNotPseudoThree(direction: Int, idx: Int, from: Int): Boolean = {
    val counters = this.collectOpen3Counters(direction, idx, b.pointFieldBlack, Flag.BLACK)
    for (idx <- counters.indices) {
      val counter = counters(idx)
      val flag = b.boardField(counter)
      if (flag != Flag.FORBIDDEN_6 && flag != Flag.FORBIDDEN_44) {
        val points = b.pointFieldBlack(counter)
        if (points.fourTotal == 0 && points.fiveTotal == 0) {
          if (points.threeTotal > 2) {
            if (this.isPseudoForbid(direction, counter, from))
              return true
          } else
            return true
        }
      }
    }

    false
  }

  private def isPseudoForbid(idx: Int): Boolean = {
    var count = 0
    val points = b.pointFieldBlack(idx)
    for (direction <- 0 until 4)
      if (points.threeAt(direction) && this.isNotPseudoThree(direction, idx, idx))
        count += 1

    count < 2
  }

  private def isPseudoForbid(excludeDirection: Int, idx: Int, from: Int): Boolean = {
    if (idx == from) return false

    var count = 0
    val points = b.pointFieldBlack(idx)
    for (direction <- 0 until 4)
      if (direction != excludeDirection && points.threeAt(direction) && this.isNotPseudoThree(direction, idx, from))
        count += 1

    count < 2
  }

  def collectTrapPoints(): (Array[Int], Array[Int]) = {
    val threeSideTraps = new mutable.ArrayBuilder.ofInt
    val fourSideTraps = new mutable.ArrayBuilder.ofInt

    for (idx <- 0 until Renju.BOARD_SIZE) {
      if (Flag.isForbid(b.boardField(idx))) {
        val points = b.pointFieldWhite(idx)

        for (direction <- 0 until 4) {
          if (points.threeAt(direction))
            threeSideTraps.addAll(this.collectOpen3Counters(direction, idx, b.pointFieldWhite, Flag.WHITE))

          if (points.closedFourAt(direction)) {
            val counter = this.collectClosed4Counter(direction, idx, b.pointFieldWhite)
            if (counter != -1)
              fourSideTraps += counter
          }
        }
      }
    }

    (threeSideTraps.result(), fourSideTraps.result())
  }

  def calculateForbids(): Unit = {
    var di3ForbidFlag = false

    for (idx <- 0 until Renju.BOARD_SIZE) {
      val points = b.pointFieldBlack(idx)

      if (points.fiveTotal > 0)
        b.boardField(idx) = Flag.FREE
      else if (b.boardField(idx) == Flag.FORBIDDEN_6)
        b.boardField(idx) = Flag.FORBIDDEN_6
      else if (points.fourTotal > 1)
        b.boardField(idx) = Flag.FORBIDDEN_44
      else if (points.threeTotal > 1) {
        b.boardField(idx) = Flag.FORBIDDEN_33
        di3ForbidFlag = true
      }
    }

    if (di3ForbidFlag)
      for (idx <- 0 until Renju.BOARD_SIZE)
        if (b.boardField(idx) == Flag.FORBIDDEN_33 && this.isPseudoForbid(idx))
          b.boardField(idx) = Flag.FREE
  }

}
