
package jrenju.protocol

import jrenju.notation.Pos
import jrenju.protocol.Solution.{CHILD_KEY, SOLUTION_KEY}

sealed trait Solution {

  val idx: Int

  def toJSON: String

  override def toString: String = this.toJSON

}

object Solution {

  val SOLUTION_KEY: String = "solution"

  val SHORT_SOLUTION_KEY: String = "s"

  val CHILD_KEY: String = "child"

  val SHORT_CHILD_KEY: String = "c"

  def fromIterable(iterable: Iterable[Int]): Option[Solution] = {
    if (iterable.isEmpty) return Option.empty

    val leaf = SolutionLeaf(iterable.last)

    Some(
      iterable
        .dropRight(1)
        .grouped(2)
        .foldRight[Solution](leaf) { (movePair, child) =>
          SolutionNode(movePair.head, Map(movePair.last -> child))
        }
    )
  }

  def fromJSON(source: String): Option[Solution] = ???

}

final case class SolutionNode(idx: Int, child: Map[Int, Solution]) extends Solution {

  def toJSON: String =
    f"{\"$SOLUTION_KEY\": \"${Pos.fromIdx(idx).toCartesian}\", \"$CHILD_KEY\": ${child.map { case (key, value) => f"\"${Pos.fromIdx(key).toCartesian}\": $value" }.mkString("{", ", ", "}")}}"

}


final case class SolutionLeaf(idx: Int) extends Solution {

  def toJSON: String =
    f"{\"$SOLUTION_KEY\": \"${Pos.fromIdx(idx).toCartesian}\"}"

}
