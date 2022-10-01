package renju.notation

sealed trait Result

object Result {

  case class FiveInRow(winner: Color) extends Result

  case object Full extends Result

}
