package jrenju

import jrenju.Points.{emptyAttributeBool, emptyAttributeNum}

import scala.language.{implicitConversions, postfixOps}

final class PointsPair(
  val black: Points = new Points(),
  val white: Points = new Points(),
) {

  @inline def isDifference(direction: Byte, that: PointsProvidePair): Boolean =
    this.black.isDifference(direction, that.black) || this.white.isDifference(direction, that.white)

  @inline def merged(direction: Byte, that: PointsProvidePair): PointsPair =
    new PointsPair(
      this.black.merged(direction, that.black),
      this.white.merged(direction, that.white),
    )

}

object PointsPair {

  val empty = new PointsPair()

}

final class Points(
  val open3: Array[Boolean] = emptyAttributeBool,
  val closed4: Array[Int] = emptyAttributeNum,
  val open4: Array[Boolean] = emptyAttributeBool,
  val five: Array[Boolean] = emptyAttributeBool,
) {

  implicit def bool2int(cond: Boolean): Int = if (cond) 1 else 0

  val three: Int = this.open3.count(_ == true)

  def threeAt: Byte = this.open3.indexWhere(_ == true).toByte

  val closedFour: Int = this.closed4.sum

  def closedFourAt: Byte = this.closed4.indexWhere(_ > 0).toByte

  val four: Int = this.open4.count(_ == true) + this.closedFour

  def fiveInRow: Int = this.five.count(_ == true)

  @inline def isDifference(direction: Byte, that: PointsProvider): Boolean =
    this.open3(direction) != that.open3 ||
      this.closed4(direction) != that.closed4 || this.open4(direction) != that.open4 ||
      this.five(direction) != that.five

  @inline def merged(direction: Byte, that: PointsProvider): Points =
    new Points(
      this.open3.updated(direction, that.open3),
      this.closed4.updated(direction, that.closed4),
      this.open4.updated(direction, that.open4),
      this.five.updated(direction, that.five)
    )

}

object Points {

  def emptyAttributeNum: Array[Int] = Array.fill(4)(0)

  def emptyAttributeBool: Array[Boolean] = Array.fill(4)(false)

}

final class PointsProvidePair(
  val black: PointsProvider = new PointsProvider(),
  val white: PointsProvider = new PointsProvider(),
)

final class PointsProvider(
  var open3: Boolean = false,
  var closed4: Int = 0,
  var open4: Boolean = false,
  var five: Boolean = false,
) {

  implicit def bool2int(value: Boolean): Int = if (value) 1 else 0

  @inline def four: Int = this.open4 + this.closed4

}
