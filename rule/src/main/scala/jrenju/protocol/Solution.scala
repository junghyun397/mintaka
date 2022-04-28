package jrenju.protocol

import jrenju.notation.Pos

sealed abstract class Solution(val idx: Int)

final class SolutionNode(idx: Int, val child: Map[Int, Solution]) extends Solution(idx) {

  override def toString: String =
    f"{solution: ${Pos.fromIdx(idx).toCartesian}, " +
      f"child: ${child.map(kv => f"${Pos.fromIdx(kv._1).toCartesian}: ${kv._2}").mkString("{", ", ", "}")}}"

}

object SolutionNode {

  implicit class SequenceToNode(sequence: Seq[Int]) {

    def toSolution: Solution = {
      val leaf = new SolutionLeaf(this.sequence.last)

      this.sequence
        .dropRight(1)
        .grouped(2)
        .foldRight[Solution](leaf) { (movePair, child) =>
          new SolutionNode(movePair.head, Map(movePair.last -> child))
        }
    }

  }

}

final class SolutionLeaf(idx: Int) extends Solution(idx) {

  override def toString: String = f"{solution: ${Pos.fromIdx(idx).toCartesian}}"

}
