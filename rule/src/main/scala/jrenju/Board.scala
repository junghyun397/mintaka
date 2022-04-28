package jrenju

import jrenju.Board.boardOps
import jrenju.notation.{Color, Flag, Pos, RejectReason, Renju, Rotation}
import jrenju.solve.ZobristHash
import jrenju.solve.ZobristHash.IncrementHash

import scala.language.implicitConversions

class Board(
  val boardField: Array[Byte],
  val pointsField: Array[PointsPair],

  var moves: Int,
  var latestMove: Int,

  var winner: Option[Byte],

  var zobristKey: Long,
) {

  @inline private def colorRaw: Byte = (this.moves % 2).toByte

  @inline private def nextColorRaw: Byte = ((this.moves + 1) % 2).toByte

  @inline private def isNextColorBlack: Boolean = this.nextColorRaw == Flag.BLACK

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

  def makeMove(idx: Int): Board = {
    val thenBoard = boardField.updated(idx, this.nextColorRaw)
    val thenPoints = this.pointsField.updated(idx, PointsPair.empty)
    new Board(
      boardField = thenBoard,
      pointsField = thenPoints,
      moves = this.moves + 1,
      latestMove = idx,
      winner = Option.empty,
      zobristKey = this.zobristKey.incrementHash(idx, this.nextColorRaw)
    )
      .calculatePoints(idx)
      .calculateForbids()
  }

  def injectMove(idx: Int): Board = {
    this.boardField(idx) = this.nextColorRaw

    this.moves += 1
    this.zobristKey = this.zobristKey.incrementHash(idx, this.colorRaw)

    this.calculateInjectedPoints(idx)
  }

  def removeMove(idx: Int): Board = {
    this.boardField(idx) = Flag.FREE
    this.pointsField(idx).clear()

    this.moves -= 1
    this.zobristKey = ZobristHash.boardHash(this.boardField)

    this.calculateInjectedPoints(idx)
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

}

object Board {

  @inline implicit def boardOps(board: Board): BoardOps = new BoardOps(board)

  @inline implicit def forbidOps(board: Board): ForbidOps = new ForbidOps(board)

  val newBoard: Board = newBoard(Renju.BOARD_CENTER.idx)

  def newBoard(initIdx: Int): Board = new Board(
    boardField = Array.fill(Renju.BOARD_LENGTH)(Flag.FREE).updated(initIdx, Flag.BLACK),
    pointsField = Array.fill(Renju.BOARD_LENGTH)(PointsPair.empty),
    moves = 1,
    winner = Option.empty,
    latestMove = initIdx,
    zobristKey = ZobristHash.empty
  )

}
