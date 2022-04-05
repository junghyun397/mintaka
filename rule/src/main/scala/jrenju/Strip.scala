package jrenju

import jrenju.L1Strip.retrieveStripField
import utils.lang.SimpleMemo.memoize

final class AttackPoints(
  val black3: Byte, val blackC4: Byte, val blackO4: Byte, val black5: Byte,
  val white3: Byte, val whiteC4: Byte, val whiteO4: Byte, val white5: Byte,
)

sealed class Strip(val startIdx: Int, val stripField: Array[Byte])

final class L1Strip(startIdx: Int, stripField: Array[Byte]) extends Strip(startIdx, stripField) {

  def calculateL2Strip(): L2Strip = {
    val assembly = retrieveStripField(this.stripField)
    new L2Strip(this.startIdx, assembly._1, assembly._2, assembly._3)
  }

}

object L1Strip {

  // KMP Algorithm, O(1)
  private def calculateStripField(field: Array[Byte]): (Array[Byte], Array[AttackPoints], Boolean) =
    (field, Array.fill(field.length)(new AttackPoints(0, 0, 0, 0, 0, 0, 0, 0)), false)

  val retrieveStripField: Array[Byte] => (Array[Byte], Array[AttackPoints], Boolean) = memoize(this.calculateStripField)

}

final class L2Strip(startIdx: Int, stripField: Array[Byte], val attackField: Array[AttackPoints], val isEnd: Boolean)
  extends Strip(startIdx, stripField)
