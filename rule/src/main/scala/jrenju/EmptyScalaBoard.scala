//noinspection ScalaUnusedSymbol

package jrenju

import jrenju.ZobristHash.IncrementHash
import jrenju.notation._

object EmptyScalaBoard extends Board {

  val field: Array[Byte] = Array.fill(Renju.BOARD_SIZE)(Flag.FREE)

  val structFieldBlack: Array[Int] = Array.fill(Renju.BOARD_SIZE)(0)

  val structFieldWhite: Array[Int] = Array.fill(Renju.BOARD_SIZE)(0)

  val moves: Int = 0

  val lastMove: Int = 0

  var winner: Option[Byte] = Option.empty

  val hashKey: Long = ZobristHash.empty

  override val color: Color.Value = Color.EMPTY

  override val nextColor: Color.Value = Color.BLACK

  override val latestPos: Option[Pos] = Option.empty

  def validateMove(idx: Int): Option[RejectReason.Value] = Option.empty

  def makeMove(idx: Int, calculateForbid: Boolean): Board = new ScalaBoard(
    field = this.field.updated(idx, Flag.BLACK),
    structFieldBlack = this.structFieldBlack.updated(idx, 0),
    structFieldWhite = this.structFieldWhite.updated(idx, 0),
    moves = 1,
    lastMove = idx,
    winner = Option.empty,
    hashKey = this.hashKey.incrementBoardHash(idx, Flag.BLACK)
  )

}
