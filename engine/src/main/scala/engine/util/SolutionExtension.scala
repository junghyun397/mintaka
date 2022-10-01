
package engine.util

import org.bson.{BsonDocument, BsonElement, BsonInt32, Document}
import renju.protocol.Solution.{SHORT_CHILD_KEY, SHORT_SOLUTION_KEY}
import renju.protocol.{Solution, SolutionLeaf, SolutionNode}
import renju.util.ListOps.listOps

extension (s: Solution) {

  def toBinary: Array[Byte] = s.toBSON.binary

  def toBSON: BsonDocument = s match {
    case SolutionNode(idx, child) =>
      val childElements = for {
        tuple <- child
        elem = {
          val (key, value) = tuple

          BsonElement(key.toString, value.toBSON)
        }
      } yield elem

      BsonDocument(
        List(
          BsonElement(SHORT_SOLUTION_KEY, BsonInt32(idx)),
          BsonElement(SHORT_CHILD_KEY, BsonDocument(childElements.toList.asJavaList)),
        ).asJavaList
      )
    case SolutionLeaf(idx) =>
      BsonDocument(SHORT_SOLUTION_KEY, BsonInt32(idx))
  }

}

def solutionFromBinary(binary: Array[Byte]): Option[Solution] = for {
  bson <- bsonFromBinary(binary)
  solution <- solutionFromBson(bson)
} yield solution

def solutionFromBson(document: BsonDocument): Option[Solution] = Option.when(!document.isEmpty) {
  val idx = document.getInt32(SHORT_SOLUTION_KEY).intValue()

  val maybeDocument = for {
    raw <- Option(document.get(SHORT_CHILD_KEY))
    childBson = raw.asDocument()
    child = for {
      raw <- childBson.asDocument().keySet().toArray
      elem = {
        val key = raw.asInstanceOf[String].toInt
        val value = childBson.get(raw).asDocument()

        (key, solutionFromBson(value).get)
      }
    } yield elem
    if child.nonEmpty
  } yield child.toMap

  maybeDocument.fold[Solution](
    ifEmpty = SolutionLeaf(idx)
  )(
    f = SolutionNode(idx, _)
  )
}

object SolutionExtension {

  def solutionToBinary(solution: Solution): Array[Byte] = solution.toBinary

  def binaryToSolution(binary: Array[Byte]): Option[Solution] = solutionFromBinary(binary)

}
