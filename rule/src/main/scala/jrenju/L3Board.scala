package jrenju

import jrenju.notation.{Direction, Flag, Opening, Pos}

import scala.collection.mutable
import scala.language.implicitConversions

class L3Board(
  boardField: Array[Byte],
  pointsField: Array[PointsPair],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening],
  override val winner: Byte,
  private val hasDi3Forbid: Boolean,
) extends Board(boardField, pointsField, moves, latestMove, opening) with EvaluatedBoard {

  @inline private def getOffsetIdxForward(direction: Int, initRow: Int, initCol: Int, offset: Int): Int = direction match {
    case Direction.X => Pos.rowColToIdx(initRow, initCol + offset)
    case Direction.Y => Pos.rowColToIdx(initRow + offset, initCol)
    case Direction.DEG45 => Pos.rowColToIdx(initRow + offset, initCol + offset)
    case Direction.DEG315 => Pos.rowColToIdx(initRow + offset, initCol - offset)
  }

  @inline private def getOffsetIdxBackward(direction: Int, initRow: Int, initCol: Int, offset: Int): Int = direction match {
    case Direction.X => Pos.rowColToIdx(initRow, initCol - offset)
    case Direction.Y => Pos.rowColToIdx(initRow - offset, initCol)
    case Direction.DEG45 => Pos.rowColToIdx(initRow - offset, initCol - offset)
    case Direction.DEG315 => Pos.rowColToIdx(initRow - offset, initCol + offset)
  }

  private def recoverCompanions(direction: Int, idx: Int): Array[Int] = {
    val row = Pos.idxToRow(idx)
    val col = Pos.idxToCol(idx)

    val buffer = mutable.ListBuffer[Int]()

    val p2Pointer = this.getOffsetIdxBackward(direction, row, col, 2)
    val p1Pointer = this.getOffsetIdxBackward(direction, row, col, 1)
    val a1Pointer = this.getOffsetIdxForward(direction, row, col, 1)
    val a2Pointer = this.getOffsetIdxForward(direction, row, col, 2)

    // +0OO+
    if (this.boardField(a1Pointer) == Flag.BLACK && this.boardField(a2Pointer) == Flag.BLACK) {
      if (this.pointsField(p1Pointer).black.open3(direction))
        buffer.append(p1Pointer)
      val end = this.getOffsetIdxForward(direction, row, col, 3)
      if (this.pointsField(end).black.open3(direction))
        buffer.append(end)
    }

    // +OO0+
    else if (this.boardField(p1Pointer) == Flag.BLACK && this.boardField(p2Pointer) == Flag.BLACK) {
      if (this.pointsField(a1Pointer).black.open3(direction))
        buffer.append(a1Pointer)
      val start = this.getOffsetIdxBackward(direction, row, col, 3)
      if (this.pointsField(start).black.open3(direction))
        buffer.append(start)
    }

    // O0+O
    else if (this.boardField(p1Pointer) == Flag.BLACK && this.boardField(a2Pointer) == Flag.BLACK)
      buffer.append(a1Pointer)
    // O+0O
    else if (this.boardField(p2Pointer) == Flag.BLACK && this.boardField(a1Pointer) == Flag.BLACK)
      buffer.append(p1Pointer)

    // -0O+O
    else if (this.boardField(p1Pointer) == Flag.FREE && this.boardField(a1Pointer) == Flag.BLACK && this.boardField(p2Pointer) >= Flag.FREE)
      buffer.append(a2Pointer)
    // O+O0-
    else if (this.boardField(p1Pointer) == Flag.BLACK && this.boardField(p2Pointer) >= Flag.FREE && this.boardField(a1Pointer) == Flag.FREE)
      buffer.append(p2Pointer)

    // +O0O+
    else if (this.boardField(p1Pointer) == Flag.BLACK && this.boardField(a1Pointer) == Flag.BLACK) {
      if (this.pointsField(a2Pointer).black.open3(direction))
        buffer.append(a2Pointer)
      if (this.pointsField(p2Pointer).black.open3(direction))
        buffer.append(p2Pointer)
    }

    buffer.toArray
  }

  implicit def int2bool(value: Int): Boolean = if (value == 0) false else true

  private def isNotPseudoThree(direction: Int, idx: Int): Boolean = this.recoverCompanions(direction, idx)
    .count { companionIdx: Int =>
      this.boardField(companionIdx) match {
        case Flag.FORBIDDEN_6 | Flag.FORBIDDEN_44 => false
        case _ =>
          val points = this.pointsField(companionIdx).black
          if (points.four > 0) false
          else if (points.three > 2)
            this.isPseudoForbid(companionIdx, direction)
          else true
      }
    }

  private def isPseudoForbid(idx: Int): Boolean = this.pointsField(idx).black.open3
    .zipWithIndex.count { open3Direction =>
      open3Direction._1 && this.isNotPseudoThree(open3Direction._2, idx)
    } < 2

  private def isPseudoForbid(idx: Int, excludeDirection: Int): Boolean = this.pointsField(idx).black.open3
    .zipWithIndex.count { open3Direction =>
      open3Direction._2 != excludeDirection && open3Direction._1 && this.isNotPseudoThree(open3Direction._2, idx)
    } < 2

  def calculateDeepL3Board(): L3Board =
    if (this.hasDi3Forbid) {
      this.boardField.zipWithIndex
        .filter(flagIdx => flagIdx._1 == Flag.FORBIDDEN_33 && this.isPseudoForbid(flagIdx._2))
        .foreach(flagIdx => this.boardField(flagIdx._2) = Flag.FREE)

      new DeepL3Board(
        this.boardField,
        this.moves,
        this.latestMove,
        this.opening,
        this.pointsField,
        this.winner,
        this.hasDi3Forbid,
      )
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
