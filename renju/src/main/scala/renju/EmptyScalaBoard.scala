package renju

import renju.hash.HashKey
import renju.notation._

object EmptyScalaBoard extends Board {

  val field: Array[Byte] = Array.fill(Renju.BOARD_SIZE)(Flag.EMPTY)

  val structFieldBlack: Array[Int] = Array.fill(Renju.BOARD_SIZE)(0)

  val structFieldWhite: Array[Int] = Array.fill(Renju.BOARD_SIZE)(0)

  val moves: Int = 0

  val lastMove: Int = 0

  override val lastPos: Option[Pos] = Option.empty

  var winner: Option[Result] = Option.empty

  val hashKey: HashKey = HashKey.empty

  override val nextColor: Color = Color.Black

  def validateMove(move: Int): Option[InvalidKind] = Option.empty

  def makeMove(move: Int, calculateForbid: Boolean): Board = new ScalaBoard(
    field = this.field.updated(move, Flag.BLACK),
    structFieldBlack = this.structFieldBlack.updated(move, 0),
    structFieldWhite = this.structFieldWhite.updated(move, 0),
    moves = 1,
    lastMove = move,
    winner = Option.empty,
    hashKey = this.hashKey.incrementHash(move, Flag.BLACK)
  )

}
