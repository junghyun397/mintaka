package jrenju.rule

import jrenju.notation.Pos

object Renju {

  @inline val BOARD_WIDTH: Int = 15

  @inline val BOARD_MAX_IDX: Int = 14

  @inline val BOARD_LENGTH: Int = BOARD_WIDTH * BOARD_WIDTH

  @inline val BOARD_CENTER: Pos = Pos(this.BOARD_WIDTH / 2, this.BOARD_WIDTH / 2)

}
