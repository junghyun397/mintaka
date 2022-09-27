
package jrenju

import jrenju.ZobristHash.IncrementHash
import jrenju.notation.{Color, Flag, InvalidKind, Renju}

import scala.language.implicitConversions

class ScalaBoard(
  val field: Array[Byte],
  val structFieldBlack: Array[Int],
  val structFieldWhite: Array[Int],

  val moves: Int,
  val lastMove: Int,

  var winner: Option[Either[Unit, Color]],

  val hashKey: Long,
) extends Board {

  def validateMove(move: Int): Option[InvalidKind] = {
    val flag = this.field(move)

    if (this.isNextColorBlack && Flag.isForbid(flag))
      Some(InvalidKind.Forbidden)
    else if (Flag.isExist(flag))
      Some(InvalidKind.Exist)
    else
      Option.empty
  }

  def makeMove(move: Int, calculateForbid: Boolean): Board = {
    val board = new ScalaBoard(
      field = this.field.updated(move, this.nextColorFlag.raw),
      structFieldBlack = this.structFieldBlack.updated(move, 0),
      structFieldWhite = this.structFieldWhite.updated(move, 0),
      moves = this.moves + 1,
      lastMove = move,
      winner = Option.empty,
      hashKey = this.hashKey.incrementBoardHash(move, this.nextColorFlag.raw)
    )

    board.integrateStrips(board.composeStrips(move).map(_.calculateL2Strip()))

    if (calculateForbid)
      board.calculateForbids()

    if (
      this.moves == Renju.BOARD_SIZE
        || (this.isNextColorBlack && (Renju.BOARD_SIZE - board.field.count(Flag.isForbid)) < 1)
    )
      this.winner = Some(Left())

    board
  }

}

object ScalaBoard {

  implicit def boardOps(board: Board): BoardOps = new BoardOps(board)

  implicit def structOps(board: Board): StructOps = new StructOps(board)

  val newBoard: Board = EmptyScalaBoard.makeMove(Renju.BOARD_CENTER_POS)

}
