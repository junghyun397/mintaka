package renju.notation

sealed trait Color {

  val flag: Flag

  val shift: Int

}

object Color {

  case object Black extends Color {

    val flag: Flag = new Flag(Flag.BLACK)

    val shift: Int = 0

  }

  case object White extends Color {

    val flag: Flag = new Flag(Flag.WHITE)

    val shift: Int = 1

  }

  def fromFlag(flag: Byte): Option[Color] = flag match {
    case Flag.BLACK => Some(Black)
    case Flag.WHITE => Some(White)
    case _ => Option.empty
  }

}
