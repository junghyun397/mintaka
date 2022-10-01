package renju.notation

sealed trait Direction {
  
  val shift: Int
  
}

object Direction {

  case object X extends Direction { val shift = 0 }
  case object Y extends Direction { val shift = 1 }
  case object IncreaseUp extends Direction { val shift = 2 }
  case object DescentUp extends Direction { val shift = 3 }
  
  val values: Array[Direction] = Array(Direction.X, Direction.Y, Direction.IncreaseUp, Direction.DescentUp)

}
