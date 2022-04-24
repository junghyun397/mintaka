package jrenju

import jrenju.notation.{Direction, Flag, Opening, Pos, Renju}

import scala.collection.mutable
import scala.language.{implicitConversions, postfixOps}

class L3Board(
  boardField: Array[Byte],
  pointsField: Array[PointsPair],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening],
  override val winner: Byte,
  private val hasDi3Forbid: Boolean,
) extends Board(boardField, pointsField, moves, latestMove, opening) with EvaluatedBoard {

  implicit def int2bool(value: Int): Boolean = if (value == 0) false else true

  @inline private def getOffsetIdx(direction: Byte, initRow: Int, initCol: Int, offset: Int): Int = direction match {
    case Direction.X => Pos.rowColToIdx(initRow, initCol + offset)
    case Direction.Y => Pos.rowColToIdx(initRow + offset, initCol)
    case Direction.DEG45 => Pos.rowColToIdx(initRow + offset, initCol + offset)
    case Direction.DEG315 => Pos.rowColToIdx(initRow + offset, initCol - offset)
  }

  @inline private def getBoardFieldBounded(idx: Int): Byte =
    if (idx < 0) Flag.WALL
    else this.boardField(idx)

  @inline private def getPointsBounded(idx: Int, op: PointsPair => Boolean): Boolean =
    if (idx < 0) false
    else op(this.pointsField(idx))

  def recoverOpen3Companions(direction: Byte, idx: Int, op: PointsPair => Points, color: Byte): Array[Int] = {
    val row = Pos.idxToRow(idx)
    val col = Pos.idxToCol(idx)

    val builder = new mutable.ArrayBuilder.ofInt

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
      if (op(this.pointsField(p1Pointer)).open3(direction))
        builder += p1Pointer
      val end = this.getOffsetIdx(direction, row, col, 3)
      if (op(this.pointsField(end)).open3(direction))
        builder += end
    }

    // +OO0+
    else if (p1Value == color && p2Value == color) {
      if (op(this.pointsField(a1Pointer)).open3(direction))
        builder += a1Pointer
      val start = this.getOffsetIdx(direction, row, col, -3)
      if (op(this.pointsField(start)).open3(direction))
        builder += start
    }

    // O0+O
    else if (p1Value == color && a2Value == color)
      builder += a1Pointer
    // O+0O
    else if (p2Value == color && a1Value == color)
      builder += p1Pointer

    // -0O+O
    else if (
      Flag.onlyStone(p1Value) == Flag.FREE
        && a1Value == color
        && Flag.onlyStone(p2Value) == Flag.FREE
    )
      builder += a2Pointer
    // O+O0-
    else if (
      p1Value == color
        && Flag.onlyStone(p2Value) == Flag.FREE
        && Flag.onlyStone(a1Value) == Flag.FREE
    )
      builder += p2Pointer

    // +O0O+
    else if (p1Value == color && a1Value == color) {
      if (op(this.pointsField(a2Pointer)).open3(direction))
        builder += a2Pointer
      if (op(this.pointsField(p2Pointer)).open3(direction))
        builder += p2Pointer
    }

    // OO+0
    else if (Flag.onlyStone(p1Value) == Flag.FREE && p2Value == color)
      builder += p1Pointer
    // 0+OO
    else if (Flag.onlyStone(a1Value) == Flag.FREE && a2Value == color)
      builder += a1Pointer

    builder.result()
  }

  def recoverClosed4Companion(direction: Byte, idx: Int, op: PointsPair => Points): Int = {
    val row = Pos.idxToRow(idx)
    val col = Pos.idxToCol(idx)

    for (offset <- -4 to 4) {
      if (offset != 0) {
        val pointer = this.getOffsetIdx(direction, row, col, offset)
        if (this.getPointsBounded(pointer, op(_).closed4(direction)))
          return pointer
      }
    }

    -1
  }

  private def isNotPseudoThree(direction: Byte, idx: Int): Boolean = {
    for (companionIdx <- this.recoverOpen3Companions(direction, idx, _.black, Flag.BLACK)) {
      val flag = this.boardField(companionIdx)
      if (flag != Flag.FORBIDDEN_6 && flag != Flag.FORBIDDEN_44) {
        val points = this.pointsField(companionIdx).black
        if (points.four == 0 && points.fiveInRow == 0) {
          if (points.three > 2) {
            if (this.isPseudoForbid(companionIdx, direction))
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
    val open3 = this.pointsField(idx).black.open3
    for (direction <- 0 until 4)
      if (open3(direction) && this.isNotPseudoThree(direction.toByte, idx))
        count += 1

    count < 2
  }

  private def isPseudoForbid(idx: Int, excludeDirection: Int): Boolean = {
    var count = 0
    val open3 = this.pointsField(idx).black.open3
    for (direction <- 0 until 4)
      if (direction != excludeDirection && open3(direction) && this.isNotPseudoThree(direction.toByte, idx))
        count += 1

    count < 2
  }

  def collectTrapPoints(): (Array[Int], Array[Int]) = {
    val threeSideTraps = new mutable.ArrayBuilder.ofInt
    val fourSideTraps = new mutable.ArrayBuilder.ofInt

    for (idx <- 0 until Renju.BOARD_LENGTH) {
      if (Flag.isForbid(this.boardField(idx))) {
        val points = this.pointsField(idx).white

        for (direction <- 0 until 4) {
          if (points.open3(direction))
            threeSideTraps.addAll(this.recoverOpen3Companions(direction.toByte, idx, _.white, Flag.WHITE))

          if (points.closed4(direction) != 0) {
            val answer = this.recoverClosed4Companion(direction.toByte, idx, _.white)
            if (answer != -1)
              fourSideTraps += answer
          }
        }
      }
    }

    (threeSideTraps.result(), fourSideTraps.result())
  }

  def calculateDeepL3Board(): L3Board =
    if (this.hasDi3Forbid) {
      for (idx <- 0 until Renju.BOARD_LENGTH) {
        val flag = this.boardField(idx)
        if (flag == Flag.FORBIDDEN_33 && this.isPseudoForbid(idx))
          this.boardField(idx) = Flag.FREE
      }

      new DeepL3Board(this.boardField, this.moves, this.latestMove, this.opening, this.pointsField, this.winner, this.hasDi3Forbid)
    } else this

}

final class DeepL3Board(
  boardField: Array[Byte],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening],
  pointsField: Array[PointsPair],
  winner: Byte,
  hasDi3Forbid: Boolean,
) extends L3Board(boardField, pointsField, moves, latestMove, opening, winner, hasDi3Forbid)
