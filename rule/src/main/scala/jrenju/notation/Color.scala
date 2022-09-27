package jrenju.notation

sealed trait Color {

  val flag: Flag

}

object Color {

  case object Black extends Color { val flag: Flag = new Flag(Flag.BLACK) }
  case object White extends Color { val flag: Flag = new Flag(Flag.WHITE) }

  def fromColor(flag: Byte): Color = flag match {
    case Flag.BLACK => Black
    case Flag.WHITE => White
    case _ => throw new IllegalArgumentException()
  }

}
