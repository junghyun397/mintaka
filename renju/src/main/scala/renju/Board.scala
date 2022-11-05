package renju

import renju.BoardIO.BoardIOExtension
import renju.hash.HashKey
import renju.notation._
import renju.util.IterableWith

trait Board extends IterableWith[FieldStatus] {

  val field: Array[Byte]

  val structFieldBlack: Array[Int]

  val structFieldWhite: Array[Int]

  var moves: Int

  var lastMove: Int

  def lastPos: Option[Pos] = Some(Pos.fromIdx(this.lastMove))

  var winner: Option[Result]

  var hashKey: HashKey

  def isNextColorBlack: Boolean = (this.moves & 0x1) == 0x0

  def color: Color = if ((moves & 0x1) == 0x1) Color.Black else Color.White
  
  def nextColor: Color = if ((moves & 0x1) == 0x1) Color.White else Color.Black

  def colorFlag: Flag = new Flag(Flag.fromMoves(this.moves))

  def nextColorFlag: Flag = new Flag(Flag.nextFromMoves(this.moves))

  def structField(idx: Int, flag: Byte): Struct = flag match {
    case Flag.BLACK => this.structFieldBlack(idx)
    case Flag.WHITE => this.structFieldWhite(idx)
    case _ => 0
  }

  def getFieldStatus(idx: Int): FieldStatus = new FieldStatus(
    flag = Flag(this.field(idx)),
    blackStruct = Struct(this.structFieldBlack(idx)),
    whiteStruct = Struct(this.structFieldWhite(idx))
  )

  def validateMove(move: Pos): Option[InvalidKind] = this.validateMove(move.idx)

  def validateMove(move: Int): Option[InvalidKind]

  def makeMove(move: Pos): Board = this.makeMove(move.idx)

  def makeMove(move: Int): Board = this.makeMove(move, calculateForbid = true)

  def makeMove(move: Int, calculateForbid: Boolean): Board

  def insertMove(move: Int, calculateForbid: Boolean): Unit

  def deleteMove(move: Int, calculateForbid: Boolean): Unit

  def length: Int = this.field.length

  def elementAt(idx: Int): FieldStatus = this.getFieldStatus(idx)

  override def toString: String = this.boardString

}
