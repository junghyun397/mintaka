package renju.notation

import scala.language.implicitConversions

// jvm word(4bytes)
// three blockThree closedFour_1 closedFour_2 openFour five  margin
// 4bits 4bits      4bits        4bits        4bits    4bits 8bits
// 28<   24<        20<          16<          12<      8<    0<       
class Struct(val raw: Int) extends AnyVal {

  // mask: 0111 0111 0111 0111 0111 0111 0000 0000
  def merged(direction: Direction, single: Int): Struct = new Struct(
    raw = (raw & (0x7777_7700 >>> direction.shift | ~(0xFFFF_FFFF >>> direction.shift))) | (single >>> direction.shift)
  )

  def threeAt(direction: Direction): Boolean = ((raw >>> 31 - direction.shift) & 0x1) == 1

  def blockThreeAt(direction: Direction): Boolean = ((raw >>> 27 - direction.shift) & 0x1) == 1

  def closedFourAt(direction: Direction): Boolean = ((raw >>> 23 - direction.shift) & 0x1) == 1

  def openFourAt(direction: Direction): Boolean = ((raw >>> 15 - direction.shift) & 0x1) == 1

  def fiveAt(direction: Direction): Boolean = ((raw >>> 11 - direction.shift) & 0x1) == 1

  def threeTotal: Int = Integer.bitCount((raw >>> 28) & 0xF)

  def blockThreeTotal: Int = Integer.bitCount((raw >>> 24) & 0xF)

  def closedFourTotal: Int = Integer.bitCount((raw >>> 16) & 0xFF)

  def openFourTotal: Int = Integer.bitCount((raw >>> 12) & 0xF)

  def fourTotal: Int = this.closedFourTotal + this.openFourTotal

  def fiveTotal: Int = Integer.bitCount((raw >>> 8) & 0xF)

}

object Struct {

  implicit def struct(raw: Int): Struct = new Struct(raw)

  def apply(raw: Int): Struct = new Struct(raw)

  val empty: Struct = new Struct(0)

}
