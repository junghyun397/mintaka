
package jrenju

import jrenju.notation.{Direction, Flag}

import scala.language.implicitConversions

class FieldStatus(
  val flag: Flag,
  val blackStruct: Struct,
  val whiteStruct: Struct,
) {

  def apply(flag: Flag): Struct = {
    flag.raw match {
      case Flag.BLACK => this.blackStruct
      case Flag.WHITE => this.whiteStruct
      case _ => Struct.empty
    }
  }

}

// jvm word(4bytes)
// three(4bits) blockThree(4bits) closedFour_1(4bits) closedFour_2(4bits) openFour(4bits) five(4bits) -> 3bytes
class Struct(val x: Int) extends AnyVal {

  // mask: 0111 0111 0111 0111 0111 0111 0000 0000
  def merged(direction: Direction, that: Int): Int =
    (x & (0x7777_7700 >>> direction.shift | ~(0xFFFF_FFFF >>> direction.shift))) | (that >>> direction.shift)

  def threeAt(direction: Direction): Boolean = ((x >>> 31 - direction.shift) & 0x1) == 1

  def blockThreeAt(direction: Direction): Boolean = ((x >>> 27 - direction.shift) & 0x1) == 1

  def closedFourAt(direction: Direction): Boolean = ((x >>> 23 - direction.shift) & 0x1) == 1

  def openFourAt(direction: Direction): Boolean = ((x >>> 15 - direction.shift) & 0x1) == 1

  def fiveAt(direction: Direction): Boolean = ((x >>> 11 - direction.shift) & 0x1) == 1

  def threeTotal: Int = Integer.bitCount((x >>> 28) & 0xF)

  def blockThreeTotal: Int = Integer.bitCount((x >>> 24) & 0xF)

  def closedFourTotal: Int = Integer.bitCount((x >>> 16) & 0xFF)

  def openFourTotal: Int = Integer.bitCount((x >>> 12) & 0xF)

  def fourTotal: Int = this.closedFourTotal + this.openFourTotal

  def fiveTotal: Int = Integer.bitCount((x >>> 8) & 0xF)

}

object Struct {

  implicit def particleOps(particle: Int): Struct = new Struct(particle)

  val empty: Struct = new Struct(0)

}
