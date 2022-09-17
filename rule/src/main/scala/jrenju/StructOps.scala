package jrenju

import jrenju.ParticleOps.particleOps
import jrenju.notation.{Direction, Flag, Pos, Renju}

import scala.collection.mutable
import scala.language.implicitConversions

final class StructOps(val b: Board) extends AnyVal {

  @inline private def getOffsetIdx(direction: Int, initRow: Int, initCol: Int, offset: Int): Int = direction match {
    case Direction.X => Pos.rowColToIdx(initRow, initCol + offset)
    case Direction.Y => Pos.rowColToIdx(initRow + offset, initCol)
    case Direction.DEG45 => Pos.rowColToIdx(initRow + offset, initCol + offset)
    case Direction.DEG315 => Pos.rowColToIdx(initRow + offset, initCol - offset)
  }

  @inline private def getBoardFieldBounded(idx: Int): Byte =
    if (idx < 0 || idx >= Renju.BOARD_SIZE) Flag.WALL
    else b.field(idx)

  @inline private def processParticleBounded(idx: Int, process: Int => Boolean): Boolean =
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
        if (this.processParticleBounded(pointer, extract(_).closedFourAt(direction)))
          return pointer
      }
    }

    -1
  }

  private def isNotPseudoThree(direction: Int, idx: Int, from: Int): Boolean = {
    val counters = this.collectOpen3Counters(direction, idx, b.structFieldBlack, Flag.BLACK)
    for (counter <- counters) {
      val flag = b.field(counter)
      if (flag != Flag.FORBIDDEN_6 && flag != Flag.FORBIDDEN_44) {
        val particle = b.structFieldBlack(counter)
        if (particle.fourTotal == 0 && particle.fiveTotal == 0) {
          if (particle.threeTotal > 2) {
            if (this.isPseudoForbid(direction, counter, from))
              return true
          } else
            return true
        }
      }
    }

    b.structFieldBlack(idx) = b.structFieldBlack(idx).merged(direction, 0x00000000)
    false
  }

  private def isPseudoForbid(idx: Int): Boolean = {
    var count = 0
    val particle = b.structFieldBlack(idx)
    for (direction <- 0 until 4)
      if (particle.threeAt(direction) && this.isNotPseudoThree(direction, idx, idx))
        count += 1

    count < 2
  }

  private def isPseudoForbid(excludeDirection: Int, idx: Int, from: Int): Boolean = {
    if (idx == from) return false

    var count = 0
    val particle = b.structFieldBlack(idx)
    for (direction <- 0 until 4)
      if (direction != excludeDirection && particle.threeAt(direction) && this.isNotPseudoThree(direction, idx, from))
        count += 1

    count < 2
  }

  def collectTrapPoints(): (Array[Int], Array[Int]) = {
    val threeSideTraps = new mutable.ArrayBuilder.ofInt
    val fourSideTraps = new mutable.ArrayBuilder.ofInt

    for (idx <- 0 until Renju.BOARD_SIZE) {
      if (Flag.isForbid(b.field(idx))) {
        val particle = b.structFieldWhite(idx)

        for (direction <- 0 until 4) {
          if (particle.threeAt(direction))
            threeSideTraps.addAll(this.collectOpen3Counters(direction, idx, b.structFieldWhite, Flag.WHITE))

          if (particle.closedFourAt(direction)) {
            val counter = this.collectClosed4Counter(direction, idx, b.structFieldWhite)
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
      val particle = b.structFieldBlack(idx)

      if (particle.fiveTotal > 0)
        b.field(idx) = Flag.FREE
      else if (b.field(idx) == Flag.FORBIDDEN_6)
        b.field(idx) = Flag.FORBIDDEN_6
      else if (particle.fourTotal > 1)
        b.field(idx) = Flag.FORBIDDEN_44
      else if (particle.threeTotal > 1) {
        b.field(idx) = Flag.FORBIDDEN_33
        hasDi3Forbid = true
      }
    }

    if (hasDi3Forbid)
      for (idx <- 0 until Renju.BOARD_SIZE)
        if (b.field(idx) == Flag.FORBIDDEN_33 && this.isPseudoForbid(idx))
          b.field(idx) = Flag.FREE
  }

}
