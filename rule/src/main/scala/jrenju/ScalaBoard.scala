//noinspection ScalaUnusedSymbol

package jrenju

import jrenju.ZobristHash.IncrementHash
import jrenju.notation.{Flag, RejectReason, Renju}

import scala.language.implicitConversions

class ScalaBoard(
  val field: Array[Byte],
  val structFieldBlack: Array[Int],
  val structFieldWhite: Array[Int],

  val moves: Int,
  val lastMove: Int,

  var winner: Option[Byte],

  val hashKey: Long,
) extends Board {

  def validateMove(idx: Int): Option[RejectReason.Value] = {
    val flag = this.field(idx)

    if (this.isNextColorBlack && Flag.isForbid(flag))
      Some(RejectReason.FORBIDDEN)
    else if (Flag.isExist(flag))
      Some(RejectReason.EXIST)
    else
      Option.empty
  }

  def makeMove(idx: Int, calculateForbid: Boolean): Board = {
    val board = new ScalaBoard(
      field = this.field.updated(idx, this.nextColorFlag),
      structFieldBlack = this.structFieldBlack.updated(idx, 0),
      structFieldWhite = this.structFieldWhite.updated(idx, 0),
      moves = this.moves + 1,
      lastMove = idx,
      winner = Option.empty,
      hashKey = this.hashKey.incrementBoardHash(idx, this.nextColorFlag)
    )

    board.integrateStrips(board.composeStrips(idx).map(_.calculateL2Strip()))

    if (calculateForbid) board.calculateForbids()

    board
  }

}

object ScalaBoard {

  implicit def boardOps(board: Board): BoardOps = new BoardOps(board)

  implicit def structOps(board: Board): StructOps = new StructOps(board)

  val newBoard: Board = this.newBoard(Renju.BOARD_CENTER_POS.idx)

  def newBoard(initIdx: Int): Board = new ScalaBoard(
    field = Array.fill(Renju.BOARD_SIZE)(Flag.FREE).updated(initIdx, Flag.BLACK),
    structFieldBlack = Array.fill(Renju.BOARD_SIZE)(0),
    structFieldWhite = Array.fill(Renju.BOARD_SIZE)(0),
    moves = 1,
    winner = Option.empty,
    lastMove = initIdx,
    hashKey = ZobristHash.empty
  )

}
