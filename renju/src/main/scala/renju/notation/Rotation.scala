package renju.notation

sealed trait Rotation

object Rotation {

  case object Straight extends Rotation
  case object Clockwise extends Rotation
  case object CounterClockwise extends Rotation
  case object Overturn extends Rotation

}
