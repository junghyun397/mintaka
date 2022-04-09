package jrenju.notation

object Flag {

  val BLACK: Byte = 1
  val WHITE: Byte = 0

  val FREE: Byte = 2

  val FORBIDDEN_33: Byte = 3
  val FORBIDDEN_44: Byte = 4
  val FORBIDDEN_6: Byte = 5

  val WALL: Byte = 127

  object Text {

    val BLACK: Char = 79 // O
    val WHITE: Char = 88 // X

    val FREE: Char = 46 // .

    val FORBIDDEN_33: Char = 51 // 3
    val FORBIDDEN_44: Char = 52 // 4
    val FORBIDDEN_6: Char = 54 // 6

  }

  def flagToChar(flag: Byte): Char = flag match {
    case Flag.FREE => Flag.Text.FREE
    case Flag.BLACK => Flag.Text.BLACK
    case Flag.WHITE => Flag.Text.WHITE
    case Flag.FORBIDDEN_33 => Flag.Text.FORBIDDEN_33
    case Flag.FORBIDDEN_44 => Flag.Text.FORBIDDEN_44
    case Flag.FORBIDDEN_6 => Flag.Text.FORBIDDEN_6
  }

  def charToFlag(char: Char): Option[Byte] = char match {
    case Flag.Text.FREE => Option.apply(Flag.FREE)
    case Flag.Text.BLACK => Option.apply(Flag.BLACK)
    case Flag.Text.WHITE => Option.apply(Flag.WHITE)
    case Flag.Text.FORBIDDEN_33 => Option.apply(Flag.FORBIDDEN_33)
    case Flag.Text.FORBIDDEN_44 => Option.apply(Flag.FORBIDDEN_44)
    case Flag.Text.FORBIDDEN_6 => Option.apply(Flag.FORBIDDEN_6)
    case _ => Option.empty
  }

  @inline def onlyStone(flag: Byte): Byte = if (flag > Flag.FREE) Flag.FREE else flag

}
