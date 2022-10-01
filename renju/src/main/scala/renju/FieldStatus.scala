package renju

import renju.notation.Flag

class FieldStatus(
  val flag: Flag,
  val blackStruct: Struct,
  val whiteStruct: Struct,
) {

  def apply(flag: Flag): Struct = {
    flag.raw match {
      case Flag.BLACK => this.blackStruct
      case Flag.WHITE => this.whiteStruct
      case _ => Struct.empty
    }
  }

}
