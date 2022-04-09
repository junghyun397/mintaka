package jrenju

import jrenju.notation._
import jrenju.rule.Renju

class Board(val boardField: Array[Byte], val moves: Int, val latestMove: Int, val opening: Option[Opening]) {

  @inline private val colorRaw: Byte = (this.moves % 2).toByte

  @inline private val nextColorRaw: Byte = ((this.moves + 1) % 2).toByte

  @inline private val isNextColorBlack: Boolean = this.nextColorRaw == Flag.BLACK

  def color: Color.Value = Color.apply(this.colorRaw)

  def nextColor: Color.Value = Color.apply(this.nextColorRaw)

  def latestPos: Pos = Pos.fromIdx(this.latestMove)

  def validateMove(pos: Pos): Option[RejectReason.Value] = this.validateMove(pos.idx)

  def validateMove(idx: Int): Option[RejectReason.Value] = {
    val flag = this.boardField(idx)
    if (this.isNextColorBlack && flag > Flag.FREE) Option.apply(RejectReason.FORBIDDEN)
    else if (flag != Flag.FREE) Option.apply(RejectReason.EXIST)
    else Option.empty
  }

  def makeMove(pos: Pos): L1Board = this.makeMove(pos.idx)

  def makeMove(idx: Int): L1Board = {
    val thenBoard = boardField.updated(idx, nextColorRaw)
    new L1Board(
      boardField = thenBoard,
      moves = this.moves + 1,
      latestMove = idx,
      opening = if (this.moves + 1 == 3) Opening.detect(thenBoard, idx) else this.opening
    )
  }

  def injectMove(move1: Pos)(move2: Pos): L1Board = this.injectMove(move1.idx)(move2.idx)

  def injectMove(move1: Int)(move2: Int): L1Board = new L1Board(
    boardField = this.boardField
      .updated(move1, this.colorRaw)
      .updated(move2, this.colorRaw),
    moves = this.moves + 2,
    latestMove = move2,
    opening = this.opening,
  )

  override def hashCode(): Int = boardField.hashCode()

}

object Board {

  val newBoard: L1Board = newBoard(Renju.BOARD_CENTER.idx)

  def newBoard(initIdx: Int): L1Board = new L1Board(
    boardField = Array.fill[Byte](Renju.BOARD_LENGTH)(Flag.FREE).updated(initIdx, Flag.BLACK),
    moves = 1,
    latestMove = initIdx,
    opening = Option.empty,
  )

}
