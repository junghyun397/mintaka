package jrenju

import jrenju.notation.Pos
import utils.lang.Transform.IntTransform

class BitField(val elems: Array[Int]) extends AnyVal {

  def apply(idx: Int): Boolean = this.apply(Pos.idxToRow(idx), Pos.idxToCol(idx))

  def apply(row: Int, col: Int): Boolean = ((this.elems(row) << col) & 0x8000_0000) == 0x8000_0000

  def applyMaskOr(row: Int, shift: Int, mask: Int): Unit = this.elems(row) |= (mask >>> shift)

  def applyMaskAnd(row: Int, shift: Int, mask: Int): Unit = this.elems(row) &= (mask >>> shift)

  def applyMaskXor(row: Int, shift: Int, mask: Int): Unit = this.elems(row) ^= (mask >>> shift)

  def |= (other: this.type): Unit = {
    var idx = 0
    val otherWords = other.elems.length
    while (idx < otherWords) {
      elems(idx) |= other.elems(idx)
      idx += 1
    }
  }

  def &= (other: this.type): Unit = {
    var idx = 0
    val otherWords = other.elems.length
    while (idx < otherWords) {
      this.elems(idx) &= other.elems(idx)
      idx += 1
    }
  }

  def &~= (other: this.type): Unit = {
    var idx = 0
    val otherWords = other.elems.length
    while (idx < otherWords) {
      this.elems(idx) &= ~other.elems(idx)
      idx += 1
    }
  }

  def ^= (other: this.type): Unit = {
    var idx = 0
    val otherWords = other.elems.length
    while (idx < otherWords) {
      this.elems(idx) ^= other.elems(idx)
      idx += 1
    }
  }

  override def toString: String = this.elems.flatMap(_.toGroupedBinaryString.appended('\n')).mkString

}

object BitField {

  def empty(width: Int): BitField = new BitField(Array.fill(width)(0))

}
