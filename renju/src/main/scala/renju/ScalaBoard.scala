package renju

import renju.hash.HashKey
import renju.notation.{Flag, InvalidKind, Renju, Result}

import scala.language.implicitConversions

class ScalaBoard(
  val field: Array[Byte],
  val structFieldBlack: Array[Int],
  val structFieldWhite: Array[Int],

  var moves: Int,
  var lastMove: Int,

  var winner: Option[Result],

  var hashKey: HashKey,
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
      hashKey = this.hashKey.incrementHash(move, this.nextColorFlag.raw)
    )

    board.integrateStrips(board.composeStrips(move).map(_.calculateL2Strip()))

    if (calculateForbid)
      board.calculateForbids()

    if (
      board.moves == Renju.BOARD_SIZE
        || (this.isNextColorBlack && (Renju.BOARD_SIZE - board.moves - board.field.count(Flag.isForbid)) < 1)
    )
      board.winner = Some(Result.Full)

    board
  }

}

object ScalaBoard {

  implicit def boardOps(board: Board): BoardOps = new BoardOps(board)

  implicit def structOps(board: Board): StructOps = new StructOps(board)

  val newBoard: Board = EmptyScalaBoard.makeMove(Renju.BOARD_CENTER_POS)

}
