package jrenju

import jrenju.L1Strip.{resolveStripField, retrieveStripFieldSolution}
import jrenju.notation.Flag
import utils.math.FNV1a

import scala.collection.mutable

object Direction {

  val X: Byte = 0
  val Y: Byte = 1
  val DEG45: Byte = 2
  val DEG315: Byte = 3

}

final class AttackPoints(
  var open2: Int = 0, var open3: Int = 0, var closed4: Int = 0, var open4: Int = 0, var five: Int = 0,
)

sealed class Strip(val direction: Byte, val startIdx: Int)

final class L1Strip(direction: Byte, startIdx: Int, stripField: Array[Byte]) extends Strip(direction, startIdx) {

  def calculateL2Strip(): L2Strip = {
    val assembly = retrieveStripFieldSolution(this.stripField) // 4112 ms
//    val assembly = resolveStripField(this.stripField) // 5352 ms
    new L2Strip(this.direction, this.startIdx, assembly._1, assembly._2, assembly._3)
  }

}

object L1Strip {

  @inline private def isOver6(field: Array[Byte], idx: Int, color: Byte) = false

  @inline private def isOpenFour(field: Array[Byte], idx: Int, color: Byte) = false

  @inline private def isOpenThree(field: Array[Byte], idx: Int, color: Byte) = false

  @inline private def isClosedFour(field: Array[Byte], idx: Int, color: Byte) = false

  // KMP Algorithm, O(1)
  private def resolveStripField(field: Array[Byte]): (Array[(AttackPoints, AttackPoints)], Array[Byte], Byte) = {
    val attackStrip = Array.fill(field.length)(new AttackPoints(), new AttackPoints())

    var winner = Flag.FREE
    var fiveCounter = 0

    var priorFlag = Flag.WALL
    var flag = Flag.WALL
    var isSolid = false

    var pointer = 0
    while (pointer < field.length) {
      flag = field(pointer)
      isSolid = flag != Flag.FREE

      if (isSolid && flag == priorFlag && fiveCounter == 3) {
        winner = flag
      } else if (isSolid && flag == priorFlag) {
        fiveCounter += 1
      } else fiveCounter = 0

      priorFlag = flag
      pointer += 1
    }

    (Array.fill(field.length)(new AttackPoints(), new AttackPoints()), field, winner)
  }

  private val stripMemo = new mutable.HashMap[BigInt, (Array[(AttackPoints, AttackPoints)], Array[Byte], Byte)]()

  private def retrieveStripFieldSolution(field: Array[Byte]): (Array[(AttackPoints, AttackPoints)], Array[Byte], Byte) =
    this.stripMemo.getOrElseUpdate(FNV1a.hash32a(field), this.resolveStripField(field))

}

final class L2Strip(direction: Byte, startIdx: Int, val attackStrip: Array[(AttackPoints, AttackPoints)], val forbiddenMask: Array[Byte], val winner: Byte)
  extends Strip(direction, startIdx)
