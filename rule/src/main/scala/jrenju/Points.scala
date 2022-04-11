package jrenju

import jrenju.Points.empty

import scala.collection.mutable

final class PointsPair(
  val black: Points = new Points(),
  val white: Points = new Points(),
)
  extends mutable.Cloneable[PointsPair] {

  override def clone(): PointsPair = new PointsPair(this.black.clone(), this.white.clone())

}

final class Points(
  var open3: Array[Byte] = empty,
  var closed4: Array[Byte] = empty,
  var open4: Array[Byte] = empty,
  var five: Array[Byte] = empty,
) extends mutable.Cloneable[Points] {

  def four: Int = this.open4.sum + this.closed4.sum

  override def clone(): Points = new Points(open3.clone(), closed4.clone(), open4.clone(), five.clone())

}

object Points {

  def empty: Array[Byte] = Array.fill(4)(0)

}

final class PointsProvidePair(
  val black: PointsProvider = new PointsProvider(),
  val white: PointsProvider = new PointsProvider(),
)

final class PointsProvider(
  var open3: Byte = 0,
  var closed4: Byte = 0,
  var open4: Byte = 0,
  var five: Byte = 0,
)
