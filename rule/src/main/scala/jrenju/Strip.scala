package jrenju

import jrenju.L1Strip.retrieveStripFieldSolution
import jrenju.notation.Flag
import jrenju.solve.ZobristHash

import java.util.concurrent.ConcurrentHashMap
import scala.math.Numeric.IntIsIntegral.{minus, plus}

sealed class Strip(val direction: Int, val startIdx: Int)

//noinspection DuplicatedCode
final class L1Strip(direction: Int, startIdx: Int, val stripField: Array[Byte]) extends Strip(direction, startIdx) {

  @inline private def isNotOver6(mask: Int): Boolean = this.isNotOver6(mask, -1, -1)

  @inline private def isNotOver6(mask1: Int, mask2: Int): Boolean = this.isNotOver6(mask1, mask2, -1)

  private def isNotOver6(mask1: Int, mask2: Int, mask3: Int): Boolean = {
    var bridged = 0

    var pointer = 0
    while (pointer < this.stripField.length) {
      if (this.stripField(pointer) == Flag.BLACK || pointer == mask1 || pointer == mask2 || pointer == mask3)
        bridged += 1
      else
        bridged = 0

      if (bridged > 5) return false
      pointer += 1
    }

    true
  }

  private def pattern2Mutate(
    pointsStrip: Array[PointsProvidePair], forbidMask: Array[Byte],
    pointer: Int, isSolid: Boolean,
    p6Flag: Byte, p5Flag: Byte, p4Flag: Byte, p3Flag: Byte, p2Flag: Byte, p1Flag: Byte, flag: Byte,
    op: (Int, Int) => Int,
  ): Unit = {
    // check five
    // OOOO+
    if (
      !isSolid && p4Flag != Flag.FREE && p4Flag != Flag.WALL
        && p4Flag == p3Flag && p3Flag == p2Flag && p2Flag == p1Flag
    )
      if (p1Flag == Flag.WHITE)
        pointsStrip(pointer).white.five = true
      else if (this.isNotOver6(pointer))
        pointsStrip(pointer).black.five = true
      else if (p5Flag != Flag.BLACK)
        forbidMask(pointer) = Flag.FORBIDDEN_6

    // OOO+O
    if (
      isSolid && p4Flag != Flag.FREE
        && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.FREE && flag == p2Flag
    )
      if (flag == Flag.WHITE)
        pointsStrip(op(pointer, 1)).white.five = true
      else if (this.isNotOver6(op(pointer, 1)))
        pointsStrip(op(pointer, 1)).black.five = true
      else
        forbidMask(op(pointer, 1)) = Flag.FORBIDDEN_6

    // check open-4
    // -OOO+-
    if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag == Flag.FREE && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.FREE
    )
      if (p2Flag == Flag.WHITE)
        pointsStrip(op(pointer, 1)).white.open4 = true
      else if (this.isNotOver6(op(pointer, 1), op(pointer, 5)))
        if (this.isNotOver6(pointer, op(pointer, 1)))
          pointsStrip(op(pointer, 1)).black.open4 = true
        else
          pointsStrip(op(pointer, 1)).black.closed4 = 1

    // -OO+O-
    if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag == Flag.FREE && p4Flag == p3Flag && p2Flag == Flag.FREE && p3Flag == p1Flag
    )
      if (p1Flag == Flag.WHITE)
        pointsStrip(op(pointer, 2)).white.open4 = true
      else {
        val condL = this.isNotOver6(op(pointer, 2), pointer)
        val condR = this.isNotOver6(op(pointer, 2), op(pointer, 5))
        if (condL && condR)
          pointsStrip(op(pointer, 2)).black.open4 = true
        else if (condL || condR)
          pointsStrip(op(pointer, 2)).black.closed4 = 1
      }

    // check closed-4
    // -OOO-+
    if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag == Flag.FREE && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.FREE
    )
      if (p2Flag == Flag.WHITE)
        pointsStrip(pointer).white.closed4 += 1
      else if (this.isNotOver6(pointer, op(pointer, 1)))
        pointsStrip(pointer).black.closed4 += 1

    // OO++O
    if (
      isSolid && p4Flag != Flag.FREE
        && p4Flag == p3Flag && p3Flag == flag && p2Flag == Flag.FREE && p1Flag == Flag.FREE
    )
      if (flag == Flag.WHITE) {
        pointsStrip(op(pointer, 1)).white.closed4 += 1
        pointsStrip(op(pointer, 2)).white.closed4 += 1
      } else if (this.isNotOver6(op(pointer, 1), op(pointer, 2))) {
        pointsStrip(op(pointer, 1)).black.closed4 += 1
        pointsStrip(op(pointer, 2)).black.closed4 += 1
      }

    // +OO-O+
    if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag == Flag.FREE && p4Flag == p3Flag && p2Flag == Flag.FREE && p3Flag == p1Flag
    )
      if (p1Flag == Flag.WHITE) pointsStrip(pointer).white.closed4 += 1
      else {
        if (this.isNotOver6(pointer, op(pointer, 2)))
          pointsStrip(pointer).black.closed4 += 1
        if (this.isNotOver6(op(pointer, 2), op(pointer, 5)))
          pointsStrip(op(pointer, 5)).black.closed4 += 1
      }

    // XOOO++
    if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag != Flag.FREE && p5Flag != p4Flag
        && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.FREE
    )
      if (p2Flag == Flag.WHITE) {
        pointsStrip(pointer).white.closed4 += 1
        pointsStrip(op(pointer, 1)).white.closed4 += 1
      } else if (this.isNotOver6(pointer, op(pointer, 1))) {
        pointsStrip(pointer).black.closed4 += 1
        pointsStrip(op(pointer, 1)).black.closed4 += 1
      }

    // X+OOO-
    if (
      !isSolid && p3Flag != Flag.FREE
        && p5Flag != Flag.FREE && p5Flag != p3Flag
        && p4Flag == Flag.FREE && p3Flag == p2Flag && p2Flag == p1Flag
    )
      if (p2Flag == Flag.WHITE)
        pointsStrip(op(pointer, 4)).white.closed4 += 1
      else if (this.isNotOver6(pointer, op(pointer, 4)))
        pointsStrip(op(pointer, 4)).black.closed4 += 1

    // XOO+O+
    if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag != Flag.FREE && p5Flag != p4Flag
        && p4Flag == p3Flag && p2Flag == Flag.FREE && p1Flag == p3Flag
    )
      if (p1Flag == Flag.WHITE) {
        pointsStrip(pointer).white.closed4 += 1
        pointsStrip(op(pointer, 2)).white.closed4 += 1
      } else if (this.isNotOver6(pointer, op(pointer, 2))) {
        pointsStrip(pointer).black.closed4 += 1
        pointsStrip(op(pointer, 2)).black.closed4 += 1
      }

    // XO+OO+
    if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag != Flag.FREE && p5Flag != p4Flag
        && p4Flag == p2Flag && p3Flag == Flag.FREE && p2Flag == p1Flag
    )
      if (p1Flag == Flag.WHITE) {
        pointsStrip(pointer).white.closed4 += 1
        pointsStrip(op(pointer, 3)).white.closed4 += 1
      } else if (this.isNotOver6(pointer, op(pointer, 3))) {
        pointsStrip(pointer).black.closed4 += 1
        pointsStrip(op(pointer, 3)).black.closed4 += 1
      }

    // check open-3
    // !-OO++-
    if (
      !isSolid && p4Flag != Flag.FREE
        && p6Flag != p4Flag
        && p5Flag == Flag.FREE && p4Flag == p3Flag && p2Flag == Flag.FREE && p1Flag == Flag.FREE
    )
      if (p4Flag == Flag.WHITE) {
        pointsStrip(op(pointer, 1)).white.open3 = true
        pointsStrip(op(pointer, 2)).white.open3 = true
      } else if (this.isNotOver6(pointer, op(pointer, 1), op(pointer, 2))) {
        pointsStrip(op(pointer, 1)).black.open3 = true
        pointsStrip(op(pointer, 2)).black.open3 = true
      } else if (p6Flag != Flag.WHITE && this.isNotOver6(op(pointer, 5), op(pointer, 6))) {
        pointsStrip(op(pointer, 2)).black.open3 = true
      }

    // X-+OO+-
    if (
      !isSolid && p3Flag != Flag.FREE
        && p6Flag != Flag.FREE && p6Flag != p3Flag
        && p5Flag == Flag.FREE && p4Flag == Flag.FREE && p3Flag == p2Flag && p1Flag == Flag.FREE
    )
      if (p3Flag == Flag.WHITE) {
        pointsStrip(op(pointer, 1)).white.open3 = true
        pointsStrip(op(pointer, 4)).white.open3 = true
      } else if (this.isNotOver6(pointer, op(pointer, 1))) {
        pointsStrip(op(pointer, 1)).black.open3 = true
        pointsStrip(op(pointer, 4)).black.open3 = true
      }

    // !-O-O+-
    if (
      !isSolid && p4Flag != Flag.FREE
        && p6Flag != p4Flag
        && p5Flag == Flag.FREE && p4Flag == p2Flag && p3Flag == Flag.FREE && p1Flag == Flag.FREE
    )
      if (p4Flag == Flag.WHITE) {
        pointsStrip(op(pointer, 1)).white.open3 = true
      } else if (this.isNotOver6(pointer, op(pointer, 1), op(pointer, 3))) {
        pointsStrip(op(pointer, 1)).black.open3 = true
      }

    // X-O+O+-

    if (
      !isSolid && p4Flag != Flag.FREE
        && p6Flag != p4Flag
        && p1Flag == Flag.FREE && p2Flag == p4Flag && p3Flag == Flag.FREE && p5Flag == Flag.FREE
    )
      if (p4Flag == Flag.WHITE) {
        pointsStrip(op(pointer, 1)).white.open3 = true
        pointsStrip(op(pointer, 3)).white.open3 = true
      } else if (this.isNotOver6(pointer, op(pointer, 1), op(pointer, 3))) {
        pointsStrip(op(pointer, 1)).black.open3 = true
        pointsStrip(op(pointer, 3)).black.open3 = true
      }
  }

  private def calculatePoints(): (Array[PointsProvidePair], Array[Byte], Byte) = {
    val pointsStrip = Array.fill(this.stripField.length)(new PointsProvidePair())
    val forbidMask = Array.fill(this.stripField.length)(Flag.FREE)

    var winner = Flag.FREE

    // registers
    var p6Flag = Flag.WALL
    var p5Flag = Flag.WALL
    var p4Flag = Flag.WALL
    var p3Flag = Flag.WALL
    var p2Flag = Flag.WALL
    var p1Flag = Flag.WALL
    var flag = Flag.WALL

    var isSolid = false

    // >>>>>
    // p6Flag | p5Flag | p4Flag | p3Flag | p2Flag | p1Flag | Flag <- pointer
    var pointer = 0
    while (pointer < this.stripField.length) {
      flag = this.stripField(pointer)
      isSolid = flag != Flag.FREE

      // check win
      if (
        isSolid
          && p4Flag == p3Flag && p3Flag == p2Flag && p2Flag == p1Flag && p1Flag == flag
          && (flag == Flag.WHITE || this.isNotOver6(pointer))
      )
        winner = flag

      // check five
      // OO+OO
      if (
        isSolid
          && p2Flag == Flag.FREE && p4Flag == p3Flag && p3Flag == p1Flag && p1Flag == flag
      )
        if (flag == Flag.WHITE) pointsStrip(pointer - 2).white.five = true
        else if (this.isNotOver6(pointer - 2)) pointsStrip(pointer - 2).black.five = true
        else forbidMask(pointer - 2) = Flag.FORBIDDEN_6

      // check closed-4
      // O+O+O
      if (
        isSolid
          && p4Flag == p2Flag && p2Flag == flag && p3Flag == Flag.FREE && p1Flag == Flag.FREE
      )
        if (flag == Flag.WHITE) {
          pointsStrip(pointer - 1).white.closed4 += 1
          pointsStrip(pointer - 3).white.closed4 += 1
        } else if (this.isNotOver6(pointer - 1, pointer - 3)) {
          pointsStrip(pointer - 1).black.closed4 += 1
          pointsStrip(pointer - 3).black.closed4 += 1
        }

      // check open-3
      // -O++O-
      if (
        !isSolid && p1Flag != Flag.FREE
          && p5Flag == Flag.FREE && p4Flag == p1Flag && p3Flag == Flag.FREE && p2Flag == Flag.FREE
      )
        if (p1Flag == Flag.WHITE) {
          pointsStrip(pointer - 2).white.open3 = true
          pointsStrip(pointer - 3).white.open3 = true
        } else if (
          this.isNotOver6(pointer, pointer - 2, pointer - 3)
            && this.isNotOver6(pointer - 2, pointer - 3, pointer - 5)
        ) {
          pointsStrip(pointer - 2).black.open3 = true
          pointsStrip(pointer - 3).black.open3 = true
        }

      // --O+O--
      if (
        !isSolid && p4Flag != Flag.FREE
          && p6Flag == Flag.FREE && p5Flag == Flag.FREE && p4Flag == p2Flag && p3Flag == Flag.FREE && p1Flag == Flag.FREE
      )
        if (p2Flag == Flag.WHITE)
          pointsStrip(pointer - 3).white.open3 = true
        else if (!(
          !this.isNotOver6(pointer, pointer - 1, pointer - 3)
            && !this.isNotOver6(pointer - 3, pointer - 5, pointer - 6)
          ))
          pointsStrip(pointer - 3).black.open3 = true

      this.pattern2Mutate(
        pointsStrip, forbidMask,
        pointer, isSolid,
        p6Flag, p5Flag, p4Flag, p3Flag, p2Flag, p1Flag, flag,
        minus,
      )

      p6Flag = p5Flag
      p5Flag = p4Flag
      p4Flag = p3Flag
      p3Flag = p2Flag
      p2Flag = p1Flag
      p1Flag = flag

      pointer += 1
    }

    p6Flag = Flag.WALL
    p5Flag = Flag.WALL
    p3Flag = Flag.WALL
    p4Flag = Flag.WALL
    p2Flag = Flag.WALL
    p1Flag = Flag.WALL

    isSolid = false

    // <<<<<
    // pointer -> Flag | p1Flag | p2Flag | p3Flag | p4Flag | p5Flag | p6Flag
    pointer = pointsStrip.length - 1
    while (pointer >= 0) {
      flag = this.stripField(pointer)
      isSolid = flag != Flag.FREE

      this.pattern2Mutate(
        pointsStrip, forbidMask,
        pointer, isSolid,
        p6Flag, p5Flag, p4Flag, p3Flag, p2Flag, p1Flag, flag,
        plus,
      )

      p6Flag = p5Flag
      p5Flag = p4Flag
      p4Flag = p3Flag
      p3Flag = p2Flag
      p2Flag = p1Flag
      p1Flag = flag
      pointer -= 1
    }

    pointer = 0
    while (pointer < this.stripField.length) {
      if (pointsStrip(pointer).black.four > 1) {
        pointsStrip(pointer).black.closed4 = 0
        pointsStrip(pointer).black.open4 = false
        forbidMask(pointer) = Flag.FORBIDDEN_44
      }

      pointer += 1
    }

    (pointsStrip, forbidMask, winner)
  }

  def calculateL2Strip(): L2Strip = {
    val assembly = retrieveStripFieldSolution(this) // VCF * 10000 for 39,418 ms
//    val assembly = calculatePoints() // VCF * 10000 for 27,525 ms
    new L2Strip(this.direction, this.startIdx, assembly._1, assembly._2, assembly._3)
  }

}

object L1Strip {

//  private val stripMemo = new mutable.HashMap[Long, (Array[PointsProvidePair], Array[Byte], Byte)]()
  private val stripMemo = new ConcurrentHashMap[Long, (Array[PointsProvidePair], Array[Byte], Byte)]()

  private def retrieveStripFieldSolution(strip: L1Strip): (Array[PointsProvidePair], Array[Byte], Byte) = {
    val hash = ZobristHash.stripHash(strip.stripField)
    val cache = this.stripMemo.get(hash)
    if (cache == null) {
      val l2 = strip.calculatePoints()
      this.stripMemo.put(hash, l2)
      l2
    } else cache
  }

}

final class L2Strip(direction: Int, startIdx: Int, val pointsStrip: Array[PointsProvidePair], val forbidMask: Array[Byte], val winner: Byte)
  extends Strip(direction, startIdx)
