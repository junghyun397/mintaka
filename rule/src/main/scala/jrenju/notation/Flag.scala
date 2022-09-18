package jrenju.notation

// margin forbidKind(6-44-33) bound  isForbid isEmpty isBlack
// 0      000                 0      0        0       0
object Flag {

  val BLACK: Byte = 0x1 // 0000 0001
  val WHITE: Byte = 0x0 // 0000 0000

  val FREE: Byte = 0x2 // 0000 0010

  val FORBIDDEN_33: Byte = 0x16 // 0001 0110
  val FORBIDDEN_44: Byte = 0x26 // 0010 0110
  val FORBIDDEN_6: Byte = 0x46 // 0100 0110

  val WALL: Byte = 0x08 // 0000 1000

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
    case Flag.Text.FREE => Some(Flag.FREE)
    case Flag.Text.BLACK => Some(Flag.BLACK)
    case Flag.Text.WHITE => Some(Flag.WHITE)
    case Flag.Text.FORBIDDEN_33 => Some(Flag.FORBIDDEN_33)
    case Flag.Text.FORBIDDEN_44 => Some(Flag.FORBIDDEN_44)
    case Flag.Text.FORBIDDEN_6 => Some(Flag.FORBIDDEN_6)
    case _ => Option.empty
  }

  @inline def colorFlag(moves: Int): Byte = (moves & 0x01).byteValue

  @inline def nextColorFlag(moves: Int): Byte = (~moves & 0x01).byteValue

  @inline def onlyStone(flag: Byte): Byte = (flag & 0x03).byteValue

  @inline def isEmpty(flag: Byte): Boolean = (flag >>> 1 & 0x01) == 0x01

  @inline def isExist(flag: Byte): Boolean = (flag >>> 1 & 0x01) == 0x00

  @inline def isForbid(flag: Byte): Boolean = ((flag >>> 2) & 0x01) == 0x01

  @inline def isForbid(flag: Byte, color: Byte): Boolean = ((flag & (color << 2)) & 0x04) == 0x04

}
