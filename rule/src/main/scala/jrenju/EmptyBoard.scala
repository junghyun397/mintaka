package jrenju

import jrenju.notation.{Color, Flag, Pos, Renju}

object EmptyBoard extends Board(
  boardField = Array.fill(Renju.BOARD_SIZE)(Flag.FREE),
  structFieldBlack = Array.fill(Renju.BOARD_SIZE)(0),
  structFieldWhite = Array.fill(Renju.BOARD_SIZE)(0),
  moves = 0,
  latestMove = 0,
  winner = Option.empty,
  zobristKey = ZobristHash.empty
) {

  override def color: Color.Value = Color.EMPTY

  override def nextColor: Color.Value = Color.BLACK

  override def latestPos: Option[Pos] = Option.empty

}
