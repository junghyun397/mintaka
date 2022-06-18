package jrenju.protocol

import jrenju.notation.Pos

sealed abstract class Solution(val idx: Int) {

  def toBinary: Array[Byte]

}

final class SolutionNode(idx: Int, val child: Map[Int, Solution]) extends Solution(idx) {

  def toJSON: String =
    f"{\"solution\": \"${Pos.fromIdx(idx).toCartesian}\", " +
      f"\"child\": ${child.map { case (key, value) => f"\"${Pos.fromIdx(key).toCartesian}\": $value" }.mkString("{", ", ", "}")}}"

  override def toString: String = this.toJSON

  override def toBinary: Array[Byte] = ???

}

final class SolutionLeaf(idx: Int) extends Solution(idx) {

  def toJSON: String = f"{\"solution\": \"${Pos.fromIdx(idx).toCartesian}\"}"

  override def toString: String = this.toJSON

  override def toBinary: Array[Byte] = ???

}
