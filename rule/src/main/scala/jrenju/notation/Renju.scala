package jrenju.notation

object Renju {

  val BOARD_WIDTH: Int = 15

  val BOARD_MAX_IDX: Int = BOARD_WIDTH - 1

  val BOARD_LENGTH: Int = BOARD_WIDTH * BOARD_WIDTH

  val BOARD_CENTER: Pos = Pos(this.BOARD_WIDTH / 2, this.BOARD_WIDTH / 2)

}
