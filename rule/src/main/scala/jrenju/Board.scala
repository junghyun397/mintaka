package jrenju

import jrenju.BoardIO.BoardToString
import jrenju.notation._
import utils.lang.IterableWith

trait Board extends IterableWith[FieldStatus] {

  val field: Array[Byte]

  val structFieldBlack: Array[Int]

  val structFieldWhite: Array[Int]

  val moves: Int

  val lastMove: Int

  var winner: Option[Either[Unit, Color]]

  val hashKey: Long

  def isNextColorBlack: Boolean = (this.moves & 0x01) == 0x00

  def color: Color = if (moves % 2 == 1) Color.Black else Color.White
  
  def nextColor: Color = if (moves % 2 == 1) Color.White else Color.Black

  def colorFlag: Flag = new Flag(Flag.fromMoves(this.moves))

  def nextColorFlag: Flag = new Flag(Flag.nextFromMoves(this.moves))

  def latestPos: Option[Pos] = Some(Pos.fromIdx(this.lastMove))

  def structField(idx: Int, flag: Byte): Int = flag match {
    case Flag.BLACK => this.structFieldBlack(idx)
    case Flag.WHITE => this.structFieldWhite(idx)
    case _ => 0
  }

  def getFieldStatus(idx: Int): FieldStatus = new FieldStatus(
    flag = new Flag(this.field(idx)),
    blackStruct = new Struct(this.structFieldBlack(idx)),
    whiteStruct = new Struct(this.structFieldWhite(idx))
  )

  def validateMove(move: Pos): Option[InvalidKind] = this.validateMove(move.idx)

  def validateMove(move: Int): Option[InvalidKind]

  def makeMove(move: Pos): Board = this.makeMove(move.idx)

  def makeMove(move: Int): Board = this.makeMove(move, calculateForbid = true)

  def makeMove(move: Int, calculateForbid: Boolean): Board

  def length: Int = this.field.length

  def elementAt(idx: Int): FieldStatus = this.getFieldStatus(idx)

  override def toString: String = this.boardString

}
