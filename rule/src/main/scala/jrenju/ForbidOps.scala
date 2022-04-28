package jrenju

import jrenju.notation.{Direction, Flag, Pos, Renju}

import scala.collection.mutable
import scala.language.implicitConversions

//noinspection DuplicatedCode
final class ForbidOps(private val b: Board) extends AnyVal {

  @inline implicit private def int2bool(value: Int): Boolean = if (value == 0) false else true

  @inline private def getOffsetIdx(direction: Int, initRow: Int, initCol: Int, offset: Int): Int = direction match {
    case Direction.X => Pos.rowColToIdx(initRow, initCol + offset)
    case Direction.Y => Pos.rowColToIdx(initRow + offset, initCol)
    case Direction.DEG45 => Pos.rowColToIdx(initRow + offset, initCol + offset)
    case Direction.DEG315 => Pos.rowColToIdx(initRow + offset, initCol - offset)
  }

  @inline private def getBoardFieldBounded(idx: Int): Byte =
    if (idx < 0) Flag.WALL
    else b.boardField(idx)

  @inline private def getPointsBounded(idx: Int, op: PointsPair => Boolean): Boolean =
    if (idx < 0) false
    else op(b.pointsField(idx))

  def recoverOpen3Companions(direction: Int, idx: Int, op: PointsPair => Points, color: Byte): Array[Int] = {
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
    if (a1Value == color && a2Value == color) {
      val builder = new mutable.ArrayBuilder.ofInt

      if (op(b.pointsField(p1Pointer)).open3(direction))
        builder += p1Pointer
      val end = this.getOffsetIdx(direction, row, col, 3)
      if (op(b.pointsField(end)).open3(direction))
        builder += end

      builder.result()
    }

    // +OO0+
    else if (p1Value == color && p2Value == color) {
      val builder = new mutable.ArrayBuilder.ofInt

      if (op(b.pointsField(a1Pointer)).open3(direction))
        builder += a1Pointer
      val start = this.getOffsetIdx(direction, row, col, -3)
      if (op(b.pointsField(start)).open3(direction))
        builder += start

      builder.result()
    }

    // O0+O
    else if (p1Value == color && a2Value == color)
      Array(a1Pointer)
    // O+0O
    else if (p2Value == color && a1Value == color)
      Array(p1Pointer)

    // -0O+O
    else if (
      Flag.onlyStone(p1Value) == Flag.FREE
        && a1Value == color
        && Flag.onlyStone(p2Value) == Flag.FREE
    )
      Array(a2Pointer)

    // O+O0-
    else if (
      p1Value == color
        && Flag.onlyStone(p2Value) == Flag.FREE
        && Flag.onlyStone(a1Value) == Flag.FREE
    )
      Array(p2Pointer)

    // +O0O+
    else if (p1Value == color && a1Value == color) {
      val builder = new mutable.ArrayBuilder.ofInt

      if (op(b.pointsField(a2Pointer)).open3(direction))
        builder += a2Pointer
      if (op(b.pointsField(p2Pointer)).open3(direction))
        builder += p2Pointer

      builder.result()
    }

    // OO+0
    else if (Flag.onlyStone(p1Value) == Flag.FREE && p2Value == color)
      Array(p1Pointer)

    // 0+OO
    else if (Flag.onlyStone(a1Value) == Flag.FREE && a2Value == color)
      Array(a1Pointer)

    else
      Array.empty
  }

  def recoverClosed4Companion(direction: Int, idx: Int, op: PointsPair => Points): Int = {
    val row = Pos.idxToRow(idx)
    val col = Pos.idxToCol(idx)

    for (offset <- -4 to 4) {
      if (offset != 0) {
        val pointer = this.getOffsetIdx(direction, row, col, offset)
        if (this.getPointsBounded(pointer, op(_).closed4(direction)))
          return pointer
      }
    }

    throw new Exception()
  }

  private def isNotPseudoThree(direction: Int, idx: Int): Boolean = {
    val companions = this.recoverOpen3Companions(direction, idx, _.black, Flag.BLACK)
    for (idx <- companions.indices) {
      val companion = companions(idx)
      val flag = b.boardField(companion)
      if (flag != Flag.FORBIDDEN_6 && flag != Flag.FORBIDDEN_44) {
        val points = b.pointsField(companion).black
        if (points.four == 0 && points.fiveInRow == 0) {
          if (points.three > 2) {
            if (this.isPseudoForbid(companion, direction))
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
    val open3 = b.pointsField(idx).black.open3
    for (direction <- 0 until 4)
      if (open3(direction) && this.isNotPseudoThree(direction, idx))
        count += 1

    count < 2
  }

  private def isPseudoForbid(idx: Int, excludeDirection: Int): Boolean = {
    var count = 0
    val open3 = b.pointsField(idx).black.open3
    for (direction <- 0 until 4)
      if (direction != excludeDirection && open3(direction) && this.isNotPseudoThree(direction, idx))
        count += 1

    count < 2
  }

  def collectTrapPoints(): (Array[Int], Array[Int]) = {
    val threeSideTraps = new mutable.ArrayBuilder.ofInt
    val fourSideTraps = new mutable.ArrayBuilder.ofInt

    for (idx <- 0 until Renju.BOARD_LENGTH) {
      if (Flag.isForbid(b.boardField(idx))) {
        val points = b.pointsField(idx).white

        for (direction <- 0 until 4) {
          if (points.open3(direction))
            threeSideTraps.addAll(this.recoverOpen3Companions(direction, idx, _.white, Flag.WHITE))

          if (points.closed4(direction) != 0)
            fourSideTraps += this.recoverClosed4Companion(direction, idx, _.white)
        }
      }
    }

    (threeSideTraps.result(), fourSideTraps.result())
  }

  def calculateForbids(calculateDeepForbid: Boolean = true): Board = {
    var di3ForbidFlag = false

    for (idx <- 0 until Renju.BOARD_LENGTH) {
      val points = b.pointsField(idx).black

      if (points.fiveInRow > 0)
        b.boardField(idx) = Flag.FREE
      else if (b.boardField(idx) == Flag.FORBIDDEN_6)
        b.boardField(idx) = Flag.FORBIDDEN_6
      else if (points.four > 1)
        b.boardField(idx) = Flag.FORBIDDEN_44
      else if (points.three > 1) {
        b.boardField(idx) = Flag.FORBIDDEN_33
        di3ForbidFlag = true
      }
    }

    if (calculateDeepForbid || di3ForbidFlag)
      for (idx <- 0 until Renju.BOARD_LENGTH) {
        val flag = b.boardField(idx)
        if (flag == Flag.FORBIDDEN_33 && this.isPseudoForbid(idx))
          b.boardField(idx) = Flag.FREE
      }

    b
  }

}
