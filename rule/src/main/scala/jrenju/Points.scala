package jrenju

import jrenju.Points.{emptyAttributeBool, emptyAttributeNum}
import utils.lang.Transform.BoolTransform

import scala.language.{implicitConversions, postfixOps}

final class PointsPair(
  val black: Points = new Points(),
  val white: Points = new Points(),
) {

  @inline def isDifference(direction: Int, that: PointsProvidePair): Boolean =
    this.black.isDifference(direction, that.black) || this.white.isDifference(direction, that.white)

  @inline def merged(direction: Int, that: PointsProvidePair): PointsPair =
    new PointsPair(
      this.black.merged(direction, that.black),
      this.white.merged(direction, that.white),
    )

}

object PointsPair {

  def empty = new PointsPair()

}

final class Points(
  val open3: Array[Boolean] = emptyAttributeBool,
  val block3: Array[Boolean] = emptyAttributeBool,
  val closed4: Array[Int] = emptyAttributeNum,
  val open4: Array[Boolean] = emptyAttributeBool,
  val five: Array[Boolean] = emptyAttributeBool,
) {

  val three: Int = this.open3(0).toInt + this.open3(1).toInt + this.open3(2).toInt + this.open3(3).toInt

  val closedFour: Int = this.closed4(0) + this.closed4(1) + this.closed4(2) + this.closed4(3)

  val four: Int = this.open4(0).toInt + this.open4(1).toInt + this.open4(2).toInt + this.open4(3).toInt + this.closedFour

  val fiveInRow: Int = this.five(0).toInt + this.five(1).toInt + this.five(2).toInt + this.five(3).toInt

  @inline def isDifference(direction: Int, that: PointsProvider): Boolean =
    this.open3(direction) != that.open3 ||
      this.closed4(direction) != that.closed4 || this.open4(direction) != that.open4 ||
      this.five(direction) != that.five

  @inline def merged(direction: Int, that: PointsProvider): Points =
    new Points(
      this.open3.updated(direction, that.open3),
      this.block3.updated(direction, that.block3),
      this.closed4.updated(direction, that.closed4),
      this.open4.updated(direction, that.open4),
      this.five.updated(direction, that.five)
    )

}

object Points {

  @inline def emptyAttributeNum: Array[Int] = Array.fill(4)(0)

  @inline def emptyAttributeBool: Array[Boolean] = Array.fill(4)(false)

}

final class PointsProvidePair(
  val black: PointsProvider = new PointsProvider(),
  val white: PointsProvider = new PointsProvider(),
)

final class PointsProvider(
  var open3: Boolean = false,
  var block3: Boolean = false,
  var closed4: Int = 0,
  var open4: Boolean = false,
  var five: Boolean = false,
) {

  @inline def four: Int = this.open4.toInt + this.closed4.toInt

}
