package jrenju.notation

object Flag {

  val BLACK: Byte = 1
  val WHITE: Byte = 0

  val FREE: Byte = 2

  val FORBIDDEN_33: Byte = 3
  val FORBIDDEN_44: Byte = 4
  val FORBIDDEN_6: Byte = 5

  val WALL: Byte = 9

  object Text {

    val BLACK: Char = 'X'
    val WHITE: Char = 'O'

    val FREE: Char = '.'

    val FORBIDDEN_33: Char = '3'
    val FORBIDDEN_44: Char = '4'
    val FORBIDDEN_6: Char = '6'

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
    case Flag.Text.FREE => Option(Flag.FREE)
    case Flag.Text.BLACK => Option(Flag.BLACK)
    case Flag.Text.WHITE => Option(Flag.WHITE)
    case Flag.Text.FORBIDDEN_33 => Option(Flag.FORBIDDEN_33)
    case Flag.Text.FORBIDDEN_44 => Option(Flag.FORBIDDEN_44)
    case Flag.Text.FORBIDDEN_6 => Option(Flag.FORBIDDEN_6)
    case _ => Option.empty
  }

  @inline def onlyStone(flag: Byte): Byte = if (this.isForbid(flag)) Flag.FREE else flag

  @inline def isEmpty(flag: Byte): Boolean = flag != Flag.BLACK && flag != Flag.WHITE

  @inline def isExist(flag: Byte): Boolean = flag == Flag.BLACK || flag == Flag.WHITE

  @inline def isForbid(flag: Byte): Boolean = flag == Flag.FORBIDDEN_33 || flag == Flag.FORBIDDEN_44 || flag == Flag.FORBIDDEN_6

}
