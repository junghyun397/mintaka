//noinspection ScalaUnusedSymbol

package jrenju.protocol

import jrenju.notation.Pos
import jrenju.protocol.Solution.{CHILD_KEY, SHORT_CHILD_KEY, SHORT_SOLUTION_KEY, SOLUTION_KEY}
import org.bson.{BsonDocument, BsonElement, BsonInt32, Document}
import utils.lang.BsonDocumentOps
import utils.lang.BsonDocumentOps.bsonDocumentOps
import utils.lang.ListOps.listOps

sealed trait Solution {

  val idx: Int

  def toJSON: String

  def toBSON: BsonDocument

  def toBinary: Array[Byte] = this.toBSON.binary

  override def toString: String = this.toJSON

}

object Solution {

  val SOLUTION_KEY: String = "solution"

  val SHORT_SOLUTION_KEY: String = "s"

  val CHILD_KEY: String = "child"

  val SHORT_CHILD_KEY: String = "c"

  def fromJSON(json: String): Option[Solution] = this.fromDocument(Document.parse(json))

  def fromBinary(binary: Array[Byte]): Option[Solution] = this.fromBsonDocument(BsonDocumentOps.fromBinary(binary))

  def fromIterable(iterable: Iterable[Int]): Option[Solution] = {
    if (iterable.isEmpty) return Option.empty

    val leaf = new SolutionLeaf(iterable.last)

    Some(
      iterable
        .dropRight(1)
        .grouped(2)
        .foldRight[Solution](leaf) { (movePair, child) =>
          new SolutionNode(movePair.head, Map(movePair.last -> child))
        }
    )
  }

  private def fromDocument(document: Document): Option[Solution] = Option.when(!document.isEmpty) {
    val idx = Pos.fromCartesian(document.getString(SOLUTION_KEY)).get.idx

    Option(document.get(CHILD_KEY))
      .map { rawDocument =>
        val childDocument = rawDocument.asInstanceOf[Document]

        childDocument.keySet().toArray
          .map { rawKey =>
            val key = Pos.fromCartesian(rawKey.asInstanceOf[String]).get.idx
            val value = childDocument.get(rawKey).asInstanceOf[Document]

            (key, this.fromDocument(value).get)
          }
          .toMap
      }
      .filter { _.nonEmpty }
      .fold[Solution](
        ifEmpty = new SolutionLeaf(idx)
      )(
        f = new SolutionNode(idx, _)
      )
  }

  private def fromBsonDocument(document: BsonDocument): Option[Solution] = Option.when(!document.isEmpty) {
    val idx = document.getInt32(SHORT_SOLUTION_KEY).intValue()

    Option(document.get(SHORT_CHILD_KEY))
      .map { rawDocument =>
        val childDocument = rawDocument.asDocument()

        childDocument.asDocument().keySet().toArray
          .map { rawKey =>
            val key = rawKey.asInstanceOf[String].toInt
            val value = childDocument.get(rawKey).asDocument()

            (key, this.fromBsonDocument(value).get)
          }
          .toMap
      }
      .filter { _.nonEmpty }
      .fold[Solution](
        ifEmpty = new SolutionLeaf(idx)
      )(
        f = new SolutionNode(idx, _)
      )
  }

}

final class SolutionNode(val idx: Int, val child: Map[Int, Solution]) extends Solution {

  def toJSON: String =
    f"{\"$SOLUTION_KEY\": \"${Pos.fromIdx(idx).toCartesian}\", \"$CHILD_KEY\": ${child.map { case (key, value) => f"\"${Pos.fromIdx(key).toCartesian}\": $value" }.mkString("{", ", ", "}")}}"

  def toBSON: BsonDocument = {
    val childElements: java.util.List[BsonElement] = this.child
      .map { case (key, value) =>
        new BsonElement(key.toString, value.toBSON)
      }
      .toList
      .asJavaList

    new BsonDocument(
      java.util.List.of(
        new BsonElement(SHORT_SOLUTION_KEY, new BsonInt32(idx)),
        new BsonElement(SHORT_CHILD_KEY, new BsonDocument(childElements)),
      )
    )
  }

}


final class SolutionLeaf(val idx: Int) extends Solution {

  def toJSON: String =
    f"{\"$SOLUTION_KEY\": \"${Pos.fromIdx(idx).toCartesian}\"}"

  def toBSON: BsonDocument = new BsonDocument(SHORT_SOLUTION_KEY, new BsonInt32(idx))

}
