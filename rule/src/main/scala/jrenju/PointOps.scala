package jrenju

import scala.language.implicitConversions

// jvm word(4bytes)
// three(4bits) blockThree(4bits) closedFour_1(4bits) closedFour_2(4bits) openFour(4bits) five(4bits) -> 3bytes
final class PointOps(private val x: Int) {

  // mask: 0111 0111 0111 0111 0111 0111 0000 1111
  def merged(direction: Int, that: Int): Int =
    ((x & (0x7777770F >>> direction | 0x7777770F << -direction)) | (that >>> direction)) & 0xFFFFFF00

  def threeAt(direction: Int): Boolean = ((x >>> 31 - direction) & 0x1) == 1

  def blockThreeAt(direction: Int): Boolean = ((x >>> 27 - direction) & 0x1) == 1

  def closedFourAt(direction: Int): Boolean = ((x >>> 23 - direction) & 0x1) == 1

  def openFourAt(direction: Int): Boolean = ((x >>> 15 - direction) & 0x1) == 1

  def fiveAt(direction: Int): Boolean = ((x >>> 11 - direction) & 0x1) == 1

  def threeTotal: Int = Integer.bitCount((x >>> 28) & 0xF)

  def blockThreeTotal: Int = Integer.bitCount((x >>> 24) & 0xF)

  def closedFourTotal: Int = Integer.bitCount((x >>> 16) & 0xFF)

  def openFourTotal: Int = Integer.bitCount((x >>> 12) & 0xF)

  def fourTotal: Int = this.closedFourTotal + this.openFourTotal

  def fiveTotal: Int = Integer.bitCount((x >>> 8) & 0xF)

}

object PointOps {

  implicit def pointsOps(points: Int): PointOps = new PointOps(points)

}
