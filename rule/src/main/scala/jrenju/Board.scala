package jrenju

import jrenju.Board.boardOps
import jrenju.notation.{Color, Flag, Pos, RejectReason, Renju, Rotation}
import jrenju.solve.ZobristHash
import jrenju.solve.ZobristHash.IncrementHash

import scala.language.implicitConversions

class Board(
  val boardField: Array[Byte],
  val pointsField: Array[PointsPair],

  val moves: Int,
  val latestMove: Int,

  var winner: Option[Byte],

  var zobristKey: Long,
) extends Cloneable {

  @inline def colorRaw: Byte = (this.moves % 2).toByte

  @inline def nextColorRaw: Byte = ((this.moves + 1) % 2).toByte

  @inline def isNextColorBlack: Boolean = this.nextColorRaw == Flag.BLACK

  def color: Color.Value = Color(this.colorRaw)

  def nextColor: Color.Value = Color(this.nextColorRaw)

  def latestPos: Option[Pos] = Option(Pos.fromIdx(this.latestMove))

  def validateMove(pos: Pos): Option[RejectReason.Value] = this.validateMove(pos.idx)

  def validateMove(idx: Int): Option[RejectReason.Value] = {
    val flag = this.boardField(idx)
    if (this.isNextColorBlack && Flag.isForbid(flag))
      Option(RejectReason.FORBIDDEN)
    else if (Flag.isExist(flag))
      Option(RejectReason.EXIST)
    else
      Option.empty
  }

  def makeMove(pos: Pos): Board = this.makeMove(pos.idx)

  def makeMove(idx: Int): Board = this.makeMove(idx, calculateForbid = true)

  def makeMove(idx: Int, calculateForbid: Boolean): Board = {
    val thenField = boardField.updated(idx, this.nextColorRaw)
    val thenPoints = this.pointsField.updated(idx, PointsPair.empty)

    val board = new Board(
      boardField = thenField,
      pointsField = thenPoints,
      moves = this.moves + 1,
      latestMove = idx,
      winner = Option.empty,
      zobristKey = this.zobristKey.incrementHash(idx, this.nextColorRaw)
    )

    board.integrateStrips(board.composeStrips(idx))
    if (calculateForbid) board.calculateForbids()

    board
  }

  def rotatedKey(rotation: Rotation.Value): Long = rotation match {
    case Rotation.CLOCKWISE => 0
    case Rotation.COUNTER_CLOCKWISE => 0
    case Rotation.OVERTURN => 0
    case _ => this.zobristKey
  }

  def rotated(rotation: Rotation.Value): Board = rotation match {
    case Rotation.CLOCKWISE => this
    case Rotation.COUNTER_CLOCKWISE => this
    case Rotation.OVERTURN => this
    case _ => this
  }

  def transposedKey(): Long = this.zobristKey

  def transposed(): Board = this

  override def clone(): Board = new Board(
    boardField = this.boardField.clone(),
    pointsField = this.pointsField.clone(),
    moves = this.moves,
    latestMove = this.latestMove,
    winner = this.winner,
    zobristKey = this.zobristKey
  )

}

object Board {

  @inline implicit def boardOps(board: Board): BoardOps = new BoardOps(board)

  @inline implicit def structOps(board: Board): StructOps = new StructOps(board)

  val newBoard: Board = newBoard(Renju.BOARD_CENTER_POS.idx)

  def newBoard(initIdx: Int): Board = new Board(
    boardField = Array.fill(Renju.BOARD_LENGTH)(Flag.FREE).updated(initIdx, Flag.BLACK),
    pointsField = Array.fill(Renju.BOARD_LENGTH)(PointsPair.empty),
    moves = 1,
    winner = Option.empty,
    latestMove = initIdx,
    zobristKey = ZobristHash.empty
  )

}
