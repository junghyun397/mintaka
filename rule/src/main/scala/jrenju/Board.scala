package jrenju

import jrenju.BoardIO.BoardToText
import jrenju.notation._
import utils.lang.IterableWith

trait Board extends IterableWith[FieldStatus] {

  val field: Array[Byte]

  val structFieldBlack: Array[Int]

  val structFieldWhite: Array[Int]

  val moves: Int

  val lastMove: Int

  var winner: Option[Byte]

  val hashKey: Long

  def isNextColorBlack: Boolean = (this.moves & 0x01) == 0x00

  def colorFlag: Byte = Flag.colorFlag(this.moves)

  def nextColorFlag: Byte = Flag.nextColorFlag(this.moves)

  def color: Color.Value = Color(this.colorFlag)

  def nextColor: Color.Value = Color(this.nextColorFlag)

  def latestPos: Option[Pos] = Option(Pos.fromIdx(this.lastMove))

  def structField(idx: Int, flag: Byte): Int = flag match {
    case Flag.BLACK => this.structFieldBlack(idx)
    case Flag.WHITE => this.structFieldWhite(idx)
    case _ => 0
  }

  def getFieldStatus(idx: Int): FieldStatus = {
    val flag = this.field(idx)

    val color =
      if (Flag.isEmpty(flag)) Color.EMPTY
      else Color(flag)

    val forbidKind =
      if (Flag.isForbid(flag))
        Option(flag)
      else
        Option.empty

    new FieldStatus(color, forbidKind, new ParticleOps(this.structFieldBlack(idx)), new ParticleOps(this.structFieldWhite(idx)))
  }

  def validateMove(pos: Pos): Option[RejectReason.Value] = this.validateMove(pos.idx)

  def validateMove(idx: Int): Option[RejectReason.Value]

  def makeMove(pos: Pos): Board = this.makeMove(pos.idx)

  def makeMove(idx: Int): Board = this.makeMove(idx, calculateForbid = true)

  def makeMove(idx: Int, calculateForbid: Boolean): Board

  def maxSize: Int = this.field.length

  def elementAt(idx: Int): FieldStatus = this.getFieldStatus(idx)

  override def toString: String = this.boardText

}
