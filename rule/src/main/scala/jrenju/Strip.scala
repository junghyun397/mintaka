package jrenju

import jrenju.L1Strip.retrieveStripFieldSolution
import jrenju.PointOps.pointsOps
import jrenju.notation.Flag
import utils.lang.ConcurrentMapOps.concurrentMapOps

import java.util.concurrent.ConcurrentHashMap
import scala.language.implicitConversions
import scala.math.Numeric.IntIsIntegral.{minus, plus}

sealed class Strip(val direction: Int, val startIdx: Int, val size: Int)

final class PointProviderOps(private var xs: Array[Int]) {

  def setThree(idx: Int): Unit = this.xs(idx) |= 0x80000000

  def setBlockThree(idx: Int): Unit = this.xs(idx) |= 0x08000000

  def increaseClosedFour(idx: Int): Unit =
    if (((xs(idx) >>> 23) & 0x1) == 0)
      this.xs(idx) |= 0x00800000
    else
      this.xs(idx) |= 0x00080000

  def setOpenFour(idx: Int): Unit = this.xs(idx) |= 0x00008000

  def setFive(idx: Int): Unit = this.xs(idx) |= 0x00000800

}

//noinspection DuplicatedCode
final class L1Strip(
  direction: Int,
  startIdx: Int,
  size: Int,
  val stripField: Array[Byte]
) extends Strip(direction, startIdx, size) {

  var zobristHash: Long = ZobristHash.stripHash(this.stripField)

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

  private implicit def pointProviderOps(xs: Array[Int]): PointProviderOps = new PointProviderOps(xs)

  private def pattern2Mutate(
    pointStripBlack: Array[Int],
    pointStripWhite: Array[Int],
    forbidMask: Array[Byte],
    whiteC4MarksSingle: Array[Boolean], whiteC4MarksDouble: Array[Boolean],
    pointer: Int, isSolid: Boolean,
    p6Flag: Byte, p5Flag: Byte, p4Flag: Byte, p3Flag: Byte, p2Flag: Byte, p1Flag: Byte, flag: Byte,
    op: (Int, Int) => Int,
  ): Unit = {
    // check five
    // OOOO+
    if (
      !isSolid && p4Flag != Flag.FREE && p4Flag != Flag.WALL
        && p4Flag == p3Flag && p3Flag == p2Flag && p2Flag == p1Flag
    ) {
      if (p1Flag == Flag.WHITE)
        pointStripWhite.setFive(pointer)
      else if (this.isNotOver6(pointer))
        pointStripBlack.setFive(pointer)
      else if (p5Flag != Flag.BLACK)
        forbidMask(pointer) = Flag.FORBIDDEN_6
    }

    // OOO+O
    if (
      isSolid && p4Flag != Flag.FREE
        && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.FREE && flag == p2Flag
    ) {
      if (flag == Flag.WHITE)
        pointStripWhite.setFive(op(pointer, 1))
      else if (this.isNotOver6(op(pointer, 1)))
        pointStripBlack.setFive(op(pointer, 1))
      else
        forbidMask(op(pointer, 1)) = Flag.FORBIDDEN_6
    }

    // check open-4
    // -OOO+-
    if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag == Flag.FREE && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.FREE
    ) {
      if (p2Flag == Flag.WHITE)
        pointStripWhite.setOpenFour(op(pointer, 1))
      else if (this.isNotOver6(op(pointer, 1), op(pointer, 5)))
        if (this.isNotOver6(pointer, op(pointer, 1)))
          pointStripBlack.setOpenFour(op(pointer, 1))
        else
          pointStripBlack.increaseClosedFour(op(pointer, 1))
    }

    // -OO+O-
    else if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag == Flag.FREE && p4Flag == p3Flag && p2Flag == Flag.FREE && p3Flag == p1Flag
    ) {
      if (p1Flag == Flag.WHITE)
        pointStripWhite.setOpenFour(op(pointer, 2))
      else {
        val condL = this.isNotOver6(op(pointer, 2), pointer)
        val condR = this.isNotOver6(op(pointer, 2), op(pointer, 5))
        if (condL && condR)
          pointStripBlack.setOpenFour(op(pointer, 2))
        else if (condL || condR)
          pointStripBlack.increaseClosedFour(op(pointer, 2))
      }
    }

    // check closed-4
    // -OOO-+
    if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag == Flag.FREE && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.FREE
    ) {
      if (p2Flag == Flag.WHITE) {
        if (!whiteC4MarksDouble(op(pointer, 1)) && !whiteC4MarksDouble(pointer))
          pointStripWhite.increaseClosedFour(pointer)
      } else if (this.isNotOver6(pointer, op(pointer, 1)))
        pointStripBlack.increaseClosedFour(pointer)
    }

    // OO++O
    else if (
      isSolid && p4Flag != Flag.FREE
        && p4Flag == p3Flag && p3Flag == flag && p2Flag == Flag.FREE && p1Flag == Flag.FREE
    ) {
      if (flag == Flag.WHITE) {
        if (!whiteC4MarksDouble(op(pointer, 1)) && !whiteC4MarksDouble(op(pointer, 2))) {
          if (p5Flag != p4Flag)
            pointStripWhite.increaseClosedFour(op(pointer, 1))
          pointStripWhite.increaseClosedFour(op(pointer, 2))
          whiteC4MarksDouble(op(pointer, 1)) = true
          whiteC4MarksDouble(op(pointer, 2)) = true
        }
      } else if (this.isNotOver6(op(pointer, 1), op(pointer, 2))) {
        pointStripBlack.increaseClosedFour(op(pointer, 1))
        pointStripBlack.increaseClosedFour(op(pointer, 2))
      }
    }

    // XOOO++
    else if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag != Flag.FREE && p5Flag != p4Flag
        && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.FREE
    ) {
      if (p2Flag == Flag.WHITE) {
        if (!whiteC4MarksDouble(pointer) && !whiteC4MarksDouble(op(pointer, 1))) {
          pointStripWhite.increaseClosedFour(pointer)
          pointStripWhite.increaseClosedFour(op(pointer, 1))
          whiteC4MarksDouble(pointer) = true
          whiteC4MarksDouble(op(pointer, 1)) = true
        }
      } else if (this.isNotOver6(pointer, op(pointer, 1))) {
        pointStripBlack.increaseClosedFour(pointer)
        pointStripBlack.increaseClosedFour(op(pointer, 1))
      }
    }

    // +OO-O+
    else if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag == Flag.FREE && p4Flag == p3Flag && p2Flag == Flag.FREE && p3Flag == p1Flag
    ) {
      if (p1Flag == Flag.WHITE) {
        if (!whiteC4MarksSingle(pointer)) {
          pointStripWhite.increaseClosedFour(pointer)
          whiteC4MarksSingle(pointer) = true
        }

        if (!whiteC4MarksSingle(op(pointer, 5))) {
          pointStripWhite.increaseClosedFour(op(pointer, 5))
          whiteC4MarksSingle(op(pointer, 5)) = true
        }
      } else {
        if (this.isNotOver6(pointer, op(pointer, 2)))
          pointStripBlack.increaseClosedFour(pointer)
        if (this.isNotOver6(op(pointer, 2), op(pointer, 5)))
          pointStripBlack.increaseClosedFour(op(pointer, 5))
      }
    }

    // X+OOO-
    else if (
      !isSolid && p3Flag != Flag.FREE
        && p5Flag != Flag.FREE && p5Flag != p3Flag
        && p4Flag == Flag.FREE && p3Flag == p2Flag && p2Flag == p1Flag
    ) {
      if (p2Flag == Flag.WHITE)
        pointStripWhite.increaseClosedFour(op(pointer, 4))
      else if (this.isNotOver6(pointer, op(pointer, 4)))
        pointStripBlack.increaseClosedFour(op(pointer, 4))
    }

    // XOO+O+
    else if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag != Flag.FREE
        && p4Flag == p3Flag && p2Flag == Flag.FREE && p1Flag == p3Flag
    ) {
      if (p1Flag == Flag.WHITE) {
        pointStripWhite.increaseClosedFour(pointer)
        if (p5Flag != p4Flag)
          pointStripWhite.increaseClosedFour(op(pointer, 2))
      } else if (p5Flag != p4Flag && this.isNotOver6(pointer, op(pointer, 2))) {
        pointStripBlack.increaseClosedFour(pointer)
        pointStripBlack.increaseClosedFour(op(pointer, 2))
      }
    }

    // XO+OO+
    else if (
      !isSolid && p4Flag != Flag.FREE
        && p5Flag != Flag.FREE
        && p4Flag == p2Flag && p3Flag == Flag.FREE && p2Flag == p1Flag
    ) {
      if (p1Flag == Flag.WHITE) {
        if (!whiteC4MarksSingle(pointer)) {
          pointStripWhite.increaseClosedFour(pointer)
          whiteC4MarksSingle(pointer) = true
        }

        if (p5Flag != p4Flag && !whiteC4MarksSingle(op(pointer, 3))) {
          pointStripWhite.increaseClosedFour(op(pointer, 3))
          whiteC4MarksSingle(op(pointer, 3)) = true
        }
      } else if (p5Flag != p4Flag && this.isNotOver6(pointer, op(pointer, 3))) {
        pointStripBlack.increaseClosedFour(pointer)
        pointStripBlack.increaseClosedFour(op(pointer, 3))
      }
    }

    // check open-3
    // !-OO++-
    if (
      !isSolid && p4Flag != Flag.FREE
        && p6Flag != p4Flag
        && p5Flag == Flag.FREE && p4Flag == p3Flag && p2Flag == Flag.FREE && p1Flag == Flag.FREE
    ) {
      if (p4Flag == Flag.WHITE) {
        pointStripWhite.setThree(op(pointer, 1))
        pointStripWhite.setThree(op(pointer, 2))
      } else if (this.isNotOver6(pointer, op(pointer, 1), op(pointer, 2))) {
        pointStripBlack.setThree(op(pointer, 1))
        pointStripBlack.setThree(op(pointer, 2))
      } else if (p6Flag != Flag.WHITE && this.isNotOver6(op(pointer, 5), op(pointer, 6))) {
        pointStripBlack.setThree(op(pointer, 2))
      }
    }

    // X-+OO+-
    if (
      !isSolid && p3Flag != Flag.FREE
        && p6Flag != Flag.FREE && p6Flag != p3Flag
        && p5Flag == Flag.FREE && p4Flag == Flag.FREE && p3Flag == p2Flag && p1Flag == Flag.FREE
    ) {
      if (p3Flag == Flag.WHITE) {
        pointStripWhite.setThree(op(pointer, 1))
        pointStripWhite.setThree(op(pointer, 4))
      } else if (this.isNotOver6(pointer, op(pointer, 1))) {
        if (this.isNotOver6(pointer, op(pointer, 1), op(pointer, 4)))
          pointStripBlack.setThree(op(pointer, 1))
        pointStripBlack.setThree(op(pointer, 4))
      }
    }

    // !-O-O+-
    if (
      !isSolid && p4Flag != Flag.FREE
        && p6Flag != p4Flag
        && p5Flag == Flag.FREE && p4Flag == p2Flag && p3Flag == Flag.FREE && p1Flag == Flag.FREE
    ) {
      if (p4Flag == Flag.WHITE) {
        pointStripWhite.setThree(op(pointer, 1))
      } else if (this.isNotOver6(pointer, op(pointer, 1), op(pointer, 3))) {
        pointStripBlack.setThree(op(pointer, 1))
      }
    }

    // X-O+O+-
    if (
      !isSolid && p4Flag != Flag.FREE
        && p6Flag != p4Flag
        && p1Flag == Flag.FREE && p2Flag == p4Flag && p3Flag == Flag.FREE && p5Flag == Flag.FREE
    ) {
      if (p4Flag == Flag.WHITE) {
        pointStripWhite.setThree(op(pointer, 1))
        pointStripWhite.setThree(op(pointer, 3))
      } else if (this.isNotOver6(pointer, op(pointer, 1), op(pointer, 3))) {
        pointStripBlack.setThree(op(pointer, 1))
        pointStripBlack.setThree(op(pointer, 3))
      }
    }
  }

  private def calculatePoints(): (Array[Int], Array[Int], Array[Byte], Byte) = {
    val pointStripBlack = Array.fill(this.stripField.length)(0)
    val pointStripWhite = Array.fill(this.stripField.length)(0)

    val forbidMask = Array.fill(this.stripField.length)(Flag.FREE)

    val whiteC4MarksSingle = Array.fill(this.stripField.length)(false)
    val whiteC4MarksDouble = Array.fill(this.stripField.length)(false)

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
        if (flag == Flag.WHITE)
          pointStripWhite.setFive(pointer - 2)
        else if (this.isNotOver6(pointer - 2))
          pointStripBlack.setFive(pointer - 2)
        else
          forbidMask(pointer - 2) = Flag.FORBIDDEN_6

      // check closed-4
      // O+O+O
      if (
        isSolid
          && p4Flag == p2Flag && p2Flag == flag && p3Flag == Flag.FREE && p1Flag == Flag.FREE
      )
        if (flag == Flag.WHITE) {
          if (
            p5Flag != Flag.WHITE
              && (pointer == this.stripField.length - 1 || this.stripField(pointer + 1) != Flag.WHITE)
          ) {
            pointStripWhite.increaseClosedFour(pointer - 1)
            pointStripWhite.increaseClosedFour(pointer - 3)
          }
        } else if (this.isNotOver6(pointer - 1, pointer - 3)) {
          pointStripBlack.increaseClosedFour(pointer - 1)
          pointStripBlack.increaseClosedFour(pointer - 3)
        }

      // check open-3
      // -O++O-
      if (
        !isSolid && p1Flag != Flag.FREE
          && p5Flag == Flag.FREE && p4Flag == p1Flag && p3Flag == Flag.FREE && p2Flag == Flag.FREE
      )
        if (p1Flag == Flag.WHITE) {
          pointStripWhite.setThree(pointer - 2)
          pointStripWhite.setThree(pointer - 3)
        } else if (
          this.isNotOver6(pointer, pointer - 2, pointer - 3)
            && this.isNotOver6(pointer - 2, pointer - 3, pointer - 5)
        ) {
          pointStripBlack.setThree(pointer - 2)
          pointStripBlack.setThree(pointer - 3)
        }

      // --O+O--
      if (
        !isSolid && p4Flag != Flag.FREE
          && p6Flag == Flag.FREE && p5Flag == Flag.FREE && p4Flag == p2Flag && p3Flag == Flag.FREE && p1Flag == Flag.FREE
      )
        if (p2Flag == Flag.WHITE)
          pointStripWhite.setThree(pointer - 3)
        else if (!(
          !this.isNotOver6(pointer, pointer - 1, pointer - 3)
            && !this.isNotOver6(pointer - 3, pointer - 5, pointer - 6)
          ))
          pointStripBlack.setThree(pointer - 3)

      this.pattern2Mutate(
        pointStripBlack,
        pointStripWhite,
        forbidMask,
        whiteC4MarksSingle, whiteC4MarksDouble,
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
    pointer = this.size - 1
    while (pointer >= 0) {
      flag = this.stripField(pointer)
      isSolid = flag != Flag.FREE

      this.pattern2Mutate(
        pointStripBlack,
        pointStripWhite,
        forbidMask,
        whiteC4MarksSingle, whiteC4MarksDouble,
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
      if (pointStripBlack(pointer).fourTotal > 1)
        forbidMask(pointer) = Flag.FORBIDDEN_44

      pointer += 1
    }

    (pointStripBlack, pointStripWhite, forbidMask, winner)
  }

  def calculateL2Strip(): L2Strip = {
    val assembly = retrieveStripFieldSolution(this) // VCF * 10000 for 29,418 ms
//    val assembly = calculatePoints() // VCF * 10000 for 58,525 ms
    new L2Strip(this.direction, this.startIdx, this.size, assembly._1, assembly._2, assembly._3, assembly._4)
  }

}

object L1Strip {

  private val stripMemo = new ConcurrentHashMap[Long, (Array[Int], Array[Int], Array[Byte], Byte)]()

  private def retrieveStripFieldSolution(strip: L1Strip): (Array[Int], Array[Int], Array[Byte], Byte) =
    this.stripMemo.getOrElseUpdate(strip.zobristHash, () => strip.calculatePoints())

}

final class L2Strip(
  direction: Int,
  startIdx: Int,
  size: Int,
  val pointStripBlack: Array[Int],
  val pointStripWhite: Array[Int],
  val forbidMask: Array[Byte],
  val winner: Byte
) extends Strip(direction, startIdx, size)
