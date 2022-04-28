package jrenju

import jrenju.Points.{emptyAttributeBool, emptyAttributeNum}

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

  @inline def merge(direction: Int, that: PointsProvidePair): Unit = {
    this.black.merge(direction, that.black)
    this.white.merge(direction, that.white)
  }

  @inline def clear(): Unit = {
    this.black.clear()
    this.white.clear()
  }

}

object PointsPair {

  def empty = new PointsPair()

}

final class Points(
  val open3: Array[Boolean] = emptyAttributeBool,
  val closed4: Array[Int] = emptyAttributeNum,
  val open4: Array[Boolean] = emptyAttributeBool,
  val five: Array[Boolean] = emptyAttributeBool,
) {

  @inline implicit def bool2int(cond: Boolean): Int = if (cond) 1 else 0

  var three: Int = calculateThree()
  @inline private def calculateThree(): Int = this.bool2int(this.open3(0)) + this.open3(1) + this.open3(2) + this.open3(3)

  var closedFour: Int = calculateClosedFour()
  @inline private def calculateClosedFour(): Int = this.closed4(0) + this.closed4(1) + this.closed4(2) + this.closed4(3)

  var four: Int = calculateFour()
  @inline private def calculateFour(): Int = this.bool2int(this.open4(0)) + this.open4(1) + this.open4(2) + this.open4(3) + this.closedFour

  var fiveInRow: Int = calculateFiveInRow()
  @inline private def calculateFiveInRow(): Int = this.bool2int(this.five(0)) + this.five(1) + this.five(2) + this.five(3)

  @inline def isDifference(direction: Int, that: PointsProvider): Boolean =
    this.open3(direction) != that.open3 ||
      this.closed4(direction) != that.closed4 || this.open4(direction) != that.open4 ||
      this.five(direction) != that.five

  @inline def merged(direction: Int, that: PointsProvider): Points =
    new Points(
      this.open3.updated(direction, that.open3),
      this.closed4.updated(direction, that.closed4),
      this.open4.updated(direction, that.open4),
      this.five.updated(direction, that.five)
    )

  @inline def merge(direction: Int, that: PointsProvider): Unit = {
    this.open3(direction) = that.open3
    this.three = this.calculateThree()

    this.closed4(direction) = that.closed4
    this.closedFour = this.calculateClosedFour()

    this.open4(direction) = that.open4
    this.four = this.calculateFour()

    this.five(direction) = that.five
    this.fiveInRow = this.calculateFiveInRow()
  }

  @inline def clear(): Unit = {
    this.open3(0) = false
    this.open3(1) = false
    this.open3(2) = false
    this.open3(3) = false
    this.three = 0

    this.closed4(0) = 0
    this.closed4(1) = 0
    this.closed4(2) = 0
    this.closed4(3) = 0
    this.closedFour = 0

    this.open4(0) = false
    this.open4(1) = false
    this.open4(2) = false
    this.open4(3) = false
    this.four = 0

    this.five(0) = false
    this.five(1) = false
    this.five(2) = false
    this.five(3) = false
    this.fiveInRow = 0
  }

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
  var closed4: Int = 0,
  var open4: Boolean = false,
  var five: Boolean = false,
) {

  @inline implicit def bool2int(value: Boolean): Int = if (value) 1 else 0

  @inline def four: Int = this.open4 + this.closed4

}
