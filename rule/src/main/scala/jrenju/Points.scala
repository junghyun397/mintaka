package jrenju

import jrenju.Points.{emptyBool, emptyNum}

import scala.collection.mutable
import scala.language.{implicitConversions, postfixOps}

final class PointsPair(
  val black: Points = new Points(),
  val white: Points = new Points(),
)
  extends mutable.Cloneable[PointsPair] {

  override def clone(): PointsPair = new PointsPair(this.black.clone(), this.white.clone())

}

final class Points(
  var open3: Array[Boolean] = emptyBool,
  var closed4: Array[Int] = emptyNum,
  var open4: Array[Boolean] = emptyBool,
  var five: Array[Boolean] = emptyBool,
) extends mutable.Cloneable[Points] {

  @inline def three: Int = this.open3.count(_ == true)

  @inline def four: Int = this.open4.count(_ == true) + this.closed4.sum

  @inline def fiveInRow: Int = this.five.count(_ == true)

  def merge(direction: Byte, that: PointsProvider): Unit = {
    this.open3(direction) = that.open3
    this.closed4(direction) = that.closed4
    this.open4(direction) = that.open4
    this.five(direction) = that.five
  }

  override def clone(): Points = new Points(open3.clone(), closed4.clone(), open4.clone(), five.clone())

}

object Points {

  def emptyNum: Array[Int] = Array.fill(4)(0)

  def emptyBool: Array[Boolean] = Array.fill(4)(false)

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
