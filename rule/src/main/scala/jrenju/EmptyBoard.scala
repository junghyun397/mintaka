package jrenju

import jrenju.notation.{Color, Flag, Pos, Renju}
import jrenju.solve.ZobristHash

object EmptyBoard extends Board(
  boardField = Array.fill(Renju.BOARD_LENGTH)(Flag.FREE),
  pointsField = Array.fill(Renju.BOARD_LENGTH)(PointsPair.empty),
  moves = 0,
  latestMove = 0,
  winner = Option.empty,
  zobristKey = ZobristHash.empty
) {

  override def color: Color.Value = Color.EMPTY

  override def nextColor: Color.Value = Color.BLACK

  override def latestPos: Option[Pos] = Option.empty

}
