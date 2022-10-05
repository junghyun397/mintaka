package renju.notation

sealed trait Result {

  val flag: Flag

}

object Result {

  case class FiveInRow(winner: Color) extends Result { val flag: Flag = winner.flag }

  case object Full extends Result { val flag: Flag = Flag.EMPTY }

  def fromFlag(flag: Byte): Result = Color.fromFlag(flag)
    .map(FiveInRow)
    .getOrElse(Full)

}
