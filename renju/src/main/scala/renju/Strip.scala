package renju

import renju.L1Strip.retrieveStripFieldSolution
import renju.notation.Struct.struct
import renju.notation.{Color, Direction, Flag, Renju}
import renju.util.Extensions.ConcurrentMapExtension

import java.util.concurrent.ConcurrentHashMap
import scala.language.{implicitConversions, postfixOps}
import scala.math.Numeric.IntIsIntegral.{minus, plus}
import scala.util.Random

trait Strip {

  val direction: Direction

  val startIdx: Int

  val size: Int

}

object Strip {

  private val TABLE_SEED = 42

  private val table: Array[Long] = {
    val random = new Random(TABLE_SEED)
    Array.fill[Long](Renju.BOARD_WIDTH * 3)(random.nextLong())
  }

  val empty: Long = new Random(TABLE_SEED << 1).nextLong()

  def hash(field: Array[Byte]): Long = {
    var result = this.empty

    var flag = Flag.WALL
    for (move <- field.indices) {
      flag = field(move)
      if (flag == Flag.BLACK)
        result ^= this.table(move)
      else if (flag == Flag.WHITE)
        result ^= this.table(Renju.BOARD_WIDTH + move)
    }

    if (field.length != Renju.BOARD_WIDTH)
      for (move <- field.length until Renju.BOARD_WIDTH)
        result ^= this.table(Renju.BOARD_WIDTH * 2 + move)

    result
  }

}

private class StructProviderOps(val xs: Array[Int]) extends AnyVal {

  def setThree(idx: Int): Unit = this.xs(idx) |= 0x8000_0000

  def setBlockThree(idx: Int): Unit = this.xs(idx) |= 0x0800_0000

  def increaseClosedFour(idx: Int): Unit = this.xs(idx) |= (0x0080_0000 >> (((xs(idx) >>> 23) & 0x1) << 2))

  def setOpenFour(idx: Int): Unit = this.xs(idx) |= 0x0000_8000

  def setFive(idx: Int): Unit = this.xs(idx) |= 0x0000_0800

}

final class L2Strip(
  val direction: Direction,
  val startIdx: Int,
  val size: Int,
  val structStripBlack: Array[Int],
  val structStripWhite: Array[Int],
  val forbidMask: Array[Byte],
  val winner: Option[Color]
) extends Strip

//noinspection DuplicatedCode
final class L1Strip(
  val direction: Direction,
  val startIdx: Int,
  val size: Int,
  val stripField: Array[Byte]
) extends Strip {

  private implicit def structProviderOps(xs: Array[Int]): StructProviderOps = new StructProviderOps(xs)

  @inline private def isNotOver5(mask: Int): Boolean = this.isNotOver5(mask, -1, -1)

  @inline private def isNotOver5(mask1: Int, mask2: Int): Boolean = this.isNotOver5(mask1, mask2, -1)

  private def isNotOver5(mask1: Int, mask2: Int, mask3: Int): Boolean = {
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

  private def isNotOver5White(mask1: Int, mask2: Int, mask3: Int): Boolean = {
    var bridged = 0

    var pointer = 0
    while (pointer < this.stripField.length) {
      if (this.stripField(pointer) == Flag.WHITE || pointer == mask1 || pointer == mask2 || pointer == mask3)
        bridged += 1
      else
        bridged = 0

      if (bridged > 5) return false
      pointer += 1
    }

    true
  }

  private def pattern2Mutate(
    structStripBlack: Array[Int],
    structStripWhite: Array[Int],
    forbidMask: Array[Byte],
    whiteC4MarksSingle: Array[Boolean], whiteC4MarksDouble: Array[Boolean],
    pointer: Int, isSolid: Boolean,
    p6Flag: Byte, p5Flag: Byte, p4Flag: Byte, p3Flag: Byte, p2Flag: Byte, p1Flag: Byte, flag: Byte,
    op: (Int, Int) => Int,
  ): Unit = {
    // check five
    // OOOO+
    if (
      !isSolid && p4Flag != Flag.EMPTY && p4Flag != Flag.WALL
        && p4Flag == p3Flag && p3Flag == p2Flag && p2Flag == p1Flag
    ) {
      if (p1Flag == Flag.WHITE)
        structStripWhite.setFive(pointer)
      else if (this.isNotOver5(pointer))
        structStripBlack.setFive(pointer)
      else if (p5Flag != Flag.BLACK)
        forbidMask(pointer) = Flag.FORBIDDEN_6
    }

    // OOO+O
    if (
      isSolid && p4Flag != Flag.EMPTY
        && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.EMPTY && flag == p2Flag
    ) {
      if (flag == Flag.WHITE)
        structStripWhite.setFive(op(pointer, 1))
      else if (this.isNotOver5(op(pointer, 1)))
        structStripBlack.setFive(op(pointer, 1))
      else
        forbidMask(op(pointer, 1)) = Flag.FORBIDDEN_6
    }

    // check open-4
    // -OOO+-
    if (
      !isSolid && p4Flag != Flag.EMPTY
        && p5Flag == Flag.EMPTY && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.EMPTY
    ) {
      if (p2Flag == Flag.WHITE) {
        structStripWhite.setOpenFour(op(pointer, 1))
        structStripWhite.setBlockThree(op(pointer, 1))
      } else if (this.isNotOver5(op(pointer, 1), op(pointer, 5))) {
        if (this.isNotOver5(pointer, op(pointer, 1)))
          structStripBlack.setOpenFour(op(pointer, 1))
        else
          structStripBlack.increaseClosedFour(op(pointer, 1))
        structStripBlack.setBlockThree(op(pointer, 1))
      }
    }

    // -OO+O-
    else if (
      !isSolid && p4Flag != Flag.EMPTY
        && p5Flag == Flag.EMPTY && p4Flag == p3Flag && p2Flag == Flag.EMPTY && p3Flag == p1Flag
    ) {
      if (p1Flag == Flag.WHITE) {
        structStripWhite.setOpenFour(op(pointer, 2))
        structStripWhite.setBlockThree(op(pointer, 2))
      } else {
        val condL = this.isNotOver5(op(pointer, 2), pointer)
        val condR = this.isNotOver5(op(pointer, 2), op(pointer, 5))
        if (condL && condR) {
          structStripBlack.setOpenFour(op(pointer, 2))
          structStripBlack.setBlockThree(op(pointer, 2))
        } else if (condL || condR)
          structStripBlack.increaseClosedFour(op(pointer, 2))
      }
    }

    // check closed-4
    // -OOO-+
    if (
      !isSolid && p4Flag != Flag.EMPTY
        && p5Flag == Flag.EMPTY && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.EMPTY
    ) {
      if (p2Flag == Flag.WHITE) {
        if (!whiteC4MarksDouble(op(pointer, 1)) && !whiteC4MarksDouble(pointer))
          structStripWhite.increaseClosedFour(pointer)

        if (p6Flag != Flag.EMPTY && structStripWhite(op(pointer, 1)).openFourAt(Direction.X))
          structStripWhite.setBlockThree(pointer)
      } else if (this.isNotOver5(pointer, op(pointer, 1))) {
        structStripBlack.increaseClosedFour(pointer)

        if (
          (p6Flag != Flag.EMPTY || !this.isNotOver5(op(pointer, 5), op(pointer, 6)))
            && structStripBlack(op(pointer, 1)).openFourAt(Direction.X)
        )
          structStripBlack.setBlockThree(pointer)
      }
    }

    // OO++O
    else if (
      isSolid && p4Flag != Flag.EMPTY
        && p4Flag == p3Flag && p3Flag == flag && p2Flag == Flag.EMPTY && p1Flag == Flag.EMPTY
    ) {
      if (flag == Flag.WHITE) {
        if (!whiteC4MarksDouble(op(pointer, 1)) && !whiteC4MarksDouble(op(pointer, 2))) {
          if (p5Flag != p4Flag)
            structStripWhite.increaseClosedFour(op(pointer, 1))
          structStripWhite.increaseClosedFour(op(pointer, 2))
          whiteC4MarksDouble(op(pointer, 1)) = true
          whiteC4MarksDouble(op(pointer, 2)) = true
        }
      } else if (this.isNotOver5(op(pointer, 1), op(pointer, 2))) {
        structStripBlack.increaseClosedFour(op(pointer, 1))
        structStripBlack.increaseClosedFour(op(pointer, 2))
      }
    }

    // XOOO++
    else if (
      !isSolid && p4Flag != Flag.EMPTY
        && p5Flag != Flag.EMPTY && p5Flag != p4Flag
        && p4Flag == p3Flag && p3Flag == p2Flag && p1Flag == Flag.EMPTY
    ) {
      if (p2Flag == Flag.WHITE) {
        if (!whiteC4MarksDouble(pointer) && !whiteC4MarksDouble(op(pointer, 1))) {
          structStripWhite.increaseClosedFour(pointer)
          structStripWhite.increaseClosedFour(op(pointer, 1))
          whiteC4MarksDouble(pointer) = true
          whiteC4MarksDouble(op(pointer, 1)) = true
        }
      } else if (this.isNotOver5(pointer, op(pointer, 1))) {
        structStripBlack.increaseClosedFour(pointer)
        structStripBlack.increaseClosedFour(op(pointer, 1))
      }
    }

    // +OO-O+
    else if (
      !isSolid && p4Flag != Flag.EMPTY
        && p5Flag == Flag.EMPTY && p4Flag == p3Flag && p2Flag == Flag.EMPTY && p3Flag == p1Flag
    ) {
      if (p1Flag == Flag.WHITE) {
        if (!whiteC4MarksSingle(pointer)) {
          structStripWhite.increaseClosedFour(pointer)
          whiteC4MarksSingle(pointer) = true
        }
        structStripWhite.setBlockThree(pointer)

        if (!whiteC4MarksSingle(op(pointer, 5))) {
          structStripWhite.increaseClosedFour(op(pointer, 5))
          whiteC4MarksSingle(op(pointer, 5)) = true
        }
        structStripWhite.setBlockThree(op(pointer, 5))
      } else {
        if (this.isNotOver5(pointer, op(pointer, 2))) {
          structStripBlack.increaseClosedFour(pointer)
          structStripBlack.setBlockThree(pointer)
        }
        if (this.isNotOver5(op(pointer, 2), op(pointer, 5))) {
          structStripBlack.increaseClosedFour(op(pointer, 5))
          structStripBlack.setBlockThree(op(pointer, 5))
        }
      }
    }

    // X+OOO-
    else if (
      !isSolid && p3Flag != Flag.EMPTY
        && p5Flag != Flag.EMPTY && p5Flag != p3Flag
        && p4Flag == Flag.EMPTY && p3Flag == p2Flag && p2Flag == p1Flag
    ) {
      if (p2Flag == Flag.WHITE) {
        structStripWhite.increaseClosedFour(op(pointer, 4))
        structStripWhite.setBlockThree(op(pointer, 4))
      } else if (this.isNotOver5(pointer, op(pointer, 4))) {
        structStripBlack.increaseClosedFour(op(pointer, 4))
        structStripBlack.setBlockThree(op(pointer, 4))
      }
    }

    // XOO+O+
    else if (
      !isSolid && p4Flag != Flag.EMPTY
        && p5Flag != Flag.EMPTY
        && p4Flag == p3Flag && p2Flag == Flag.EMPTY && p1Flag == p3Flag
    ) {
      if (p1Flag == Flag.WHITE) {
        structStripWhite.increaseClosedFour(pointer)
        if (p5Flag != p4Flag)
          structStripWhite.increaseClosedFour(op(pointer, 2))
      } else if (p5Flag != p4Flag && this.isNotOver5(pointer, op(pointer, 2))) {
        structStripBlack.increaseClosedFour(pointer)
        structStripBlack.increaseClosedFour(op(pointer, 2))
      }
    }

    // XO+OO+
    else if (
      !isSolid && p4Flag != Flag.EMPTY
        && p5Flag != Flag.EMPTY
        && p4Flag == p2Flag && p3Flag == Flag.EMPTY && p2Flag == p1Flag
    ) {
      if (p1Flag == Flag.WHITE) {
        if (!whiteC4MarksSingle(pointer)) {
          structStripWhite.increaseClosedFour(pointer)
          whiteC4MarksSingle(pointer) = true
        }

        if (p5Flag != p4Flag && !whiteC4MarksSingle(op(pointer, 3))) {
          structStripWhite.increaseClosedFour(op(pointer, 3))
          whiteC4MarksSingle(op(pointer, 3)) = true
        }
      } else if (p5Flag != p4Flag && this.isNotOver5(pointer, op(pointer, 3))) {
        structStripBlack.increaseClosedFour(pointer)
        structStripBlack.increaseClosedFour(op(pointer, 3))
      }
    }

    // check open-3
    // !-OO++-
    if (
      !isSolid && p4Flag != Flag.EMPTY
        && p6Flag != p4Flag
        && p5Flag == Flag.EMPTY && p4Flag == p3Flag && p2Flag == Flag.EMPTY && p1Flag == Flag.EMPTY
    ) {
      if (p4Flag == Flag.WHITE) {
        structStripWhite.setThree(op(pointer, 1))
        structStripWhite.setThree(op(pointer, 2))
      } else if (this.isNotOver5(pointer, op(pointer, 1), op(pointer, 2))) {
        structStripBlack.setThree(op(pointer, 1))
        structStripBlack.setThree(op(pointer, 2))
      } else if (p6Flag != Flag.WHITE && this.isNotOver5(op(pointer, 5), op(pointer, 6))) {
        structStripBlack.setThree(op(pointer, 2))
      }
    }

    // X-+OO+-
    if (
      !isSolid && p3Flag != Flag.EMPTY
        && p6Flag != Flag.EMPTY && p6Flag != p3Flag
        && p5Flag == Flag.EMPTY && p4Flag == Flag.EMPTY && p3Flag == p2Flag && p1Flag == Flag.EMPTY
    ) {
      if (p3Flag == Flag.WHITE) {
        if (this.isNotOver5White(pointer, op(pointer, 1), op(pointer, 4)))
          structStripWhite.setThree(op(pointer, 1))
        structStripWhite.setThree(op(pointer, 4))
      } else if (this.isNotOver5(pointer, op(pointer, 1))) {
        if (this.isNotOver5(pointer, op(pointer, 1), op(pointer, 4)))
          structStripBlack.setThree(op(pointer, 1))
        structStripBlack.setThree(op(pointer, 4))
      }
    }

    // !-O-O+-
    if (
      !isSolid && p4Flag != Flag.EMPTY
        && p6Flag != p4Flag
        && p5Flag == Flag.EMPTY && p4Flag == p2Flag && p3Flag == Flag.EMPTY && p1Flag == Flag.EMPTY
    ) {
      if (p4Flag == Flag.WHITE) {
        structStripWhite.setThree(op(pointer, 1))
      } else if (this.isNotOver5(pointer, op(pointer, 1), op(pointer, 3))) {
        structStripBlack.setThree(op(pointer, 1))
      }
    }

    // X-O+O+-
    if (
      !isSolid && p4Flag != Flag.EMPTY
        && p6Flag != p4Flag
        && p1Flag == Flag.EMPTY && p2Flag == p4Flag && p3Flag == Flag.EMPTY && p5Flag == Flag.EMPTY
    ) {
      if (p4Flag == Flag.WHITE) {
        structStripWhite.setThree(op(pointer, 1))
        structStripWhite.setThree(op(pointer, 3))
      } else if (this.isNotOver5(pointer, op(pointer, 1), op(pointer, 3))) {
        structStripBlack.setThree(op(pointer, 1))
        structStripBlack.setThree(op(pointer, 3))
      }
    }
  }

  private def calculateStruct(): (Array[Int], Array[Int], Array[Byte], Option[Color]) = {
    val structStripBlack = new Array[Int](this.stripField.length)
    val structStripWhite = new Array[Int](this.stripField.length)

    val forbidMask = Array.fill(this.stripField.length)(Flag.EMPTY)

    val whiteC4MarksSingle = Array.fill(this.stripField.length)(false)
    val whiteC4MarksDouble = Array.fill(this.stripField.length)(false)

    var winner = Option.empty[Color]

    // flags
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
      isSolid = flag != Flag.EMPTY

      // check win
      if (
        isSolid
          && p4Flag == p3Flag && p3Flag == p2Flag && p2Flag == p1Flag && p1Flag == flag
          && (flag == Flag.WHITE || this.isNotOver5(pointer))
      )
        winner = Some(Color.fromFlag(flag).get)

      // check five
      // OO+OO
      if (
        isSolid
          && p2Flag == Flag.EMPTY && p4Flag == p3Flag && p3Flag == p1Flag && p1Flag == flag
      )
        if (flag == Flag.WHITE)
          structStripWhite.setFive(pointer - 2)
        else if (this.isNotOver5(pointer - 2))
          structStripBlack.setFive(pointer - 2)
        else
          forbidMask(pointer - 2) = Flag.FORBIDDEN_6

      // check closed-4
      // O+O+O
      if (
        isSolid
          && p4Flag == p2Flag && p2Flag == flag && p3Flag == Flag.EMPTY && p1Flag == Flag.EMPTY
      )
        if (flag == Flag.WHITE) {
          if (
            p5Flag != Flag.WHITE
              && (pointer == this.stripField.length - 1 || this.stripField(pointer + 1) != Flag.WHITE)
          ) {
            structStripWhite.increaseClosedFour(pointer - 1)
            structStripWhite.increaseClosedFour(pointer - 3)
          }
        } else if (this.isNotOver5(pointer - 1, pointer - 3)) {
          structStripBlack.increaseClosedFour(pointer - 1)
          structStripBlack.increaseClosedFour(pointer - 3)
        }

      // check open-3
      // -O++O-
      if (
        !isSolid && p1Flag != Flag.EMPTY
          && p5Flag == Flag.EMPTY && p4Flag == p1Flag && p3Flag == Flag.EMPTY && p2Flag == Flag.EMPTY
      )
        if (p1Flag == Flag.WHITE) {
          structStripWhite.setThree(pointer - 2)
          structStripWhite.setThree(pointer - 3)
        } else if (
          this.isNotOver5(pointer, pointer - 2, pointer - 3)
            && this.isNotOver5(pointer - 2, pointer - 3, pointer - 5)
        ) {
          structStripBlack.setThree(pointer - 2)
          structStripBlack.setThree(pointer - 3)
        }

      // --O+O--
      if (
        !isSolid && p4Flag != Flag.EMPTY
          && p6Flag == Flag.EMPTY && p5Flag == Flag.EMPTY && p4Flag == p2Flag && p3Flag == Flag.EMPTY && p1Flag == Flag.EMPTY
      )
        if (p2Flag == Flag.WHITE)
          structStripWhite.setThree(pointer - 3)
        else if (!(
          !this.isNotOver5(pointer, pointer - 1, pointer - 3)
            && !this.isNotOver5(pointer - 3, pointer - 5, pointer - 6)
          ))
          structStripBlack.setThree(pointer - 3)

      this.pattern2Mutate(
        structStripBlack,
        structStripWhite,
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
      isSolid = flag != Flag.EMPTY

      this.pattern2Mutate(
        structStripBlack,
        structStripWhite,
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
      if (structStripBlack(pointer).fourTotal > 1)
        forbidMask(pointer) = Flag.FORBIDDEN_44

      pointer += 1
    }

    (structStripBlack, structStripWhite, forbidMask, winner)
  }

  def calculateL2Strip(): L2Strip = {
    val assembly = retrieveStripFieldSolution(this) // VCF * 10000 for 29,418 ms
//    val assembly = calculateStruct() // VCF * 10000 for 58,525 ms
    new L2Strip(this.direction, this.startIdx, this.size, assembly._1, assembly._2, assembly._3, assembly._4)
  }

}

object L1Strip {

  type SolutionTuple = (Array[Int], Array[Int], Array[Byte], Option[Color])

  private val stripMemo = new ConcurrentHashMap[Long, SolutionTuple]()

  private def retrieveStripFieldSolution(strip: L1Strip): SolutionTuple =
    this.stripMemo.getOrElseUpdate(Strip.hash(strip.stripField), () => strip.calculateStruct())

//  private def retrieveStripFieldSolution_(strip: L1Strip): SolutionTuple = {
//    val hashKey = Strip.hash(strip.stripField)
//    val solutionRef = StripTranspositionTable.find(hashKey, strip.stripField)
//
//    if (solutionRef == null) {
//      val solution = strip.calculateStruct()
//
//      StripTranspositionTable.write(hashKey, solution)
//
//      solution
//    } else
//      solutionRef
//  }

}

//private object StripTranspositionTable {
//
//  private val BUCKET_SIZE: Int = 4
//
//  private val TABLE_SIZE: Int = 16 * 1024
//  private val TOTAL_SIZE: Int = this.TABLE_SIZE * BUCKET_SIZE
//
//  private val KEYS: Array[Long] = new Array[Long](this.TOTAL_SIZE)
//  private val INFO: Array[Int] = new Array[Int](this.TOTAL_SIZE)
//  private val REFS: Array[SolutionTuple] = new Array[SolutionTuple](this.TOTAL_SIZE)
//
//  def write(hashKey: Long, ref: SolutionTuple): Unit = {
//    val timestamp = System.currentTimeMillis().toInt
//
//    var idx = this.calculateBeginIdx(hashKey)
//    val endIdx = idx + BUCKET_SIZE
//
//    var targetIdx = -1
//    var targetTime = Int.MaxValue
//
//    var take = true
//    while (idx < endIdx && take) {
//      val key = this.KEYS(idx)
//      val ref = this.REFS(idx)
//
//      if (key == 0 && ref == null) {
//        targetIdx = idx
//        take = false
//      } else if (key == hashKey) {
//        if (this.INFO(idx) > timestamp) return
//
//        targetIdx = idx
//        take = false
//      } else {
//        if (targetTime > this.INFO(idx)) {
//          targetIdx = idx
//          targetTime = this.INFO(idx)
//        }
//
//        idx += 1
//      }
//    }
//
//    this.KEYS(targetIdx) = hashKey
//    this.REFS(targetIdx) = ref
//    this.INFO(targetIdx) = timestamp
//  }
//
//  def find(hashKey: Long, field: Array[Byte]): SolutionTuple = {
//    var idx = this.calculateBeginIdx(hashKey)
//    val endIdx = idx + BUCKET_SIZE
//
//    while (idx < endIdx) {
//      val ref = this.REFS(idx)
//      if (this.KEYS(idx) == hashKey && field.sameElements(ref._1))
//        return ref
//
//      idx += 1
//    }
//
//    null
//  }
//
//  def calculateBeginIdx(hashKey: Long): Int =
//    (hashKey % this.TABLE_SIZE).toInt.abs
//
//}
