package renju.util

import scala.collection.mutable

object Transform {

  def joinHorizontal(elems: String*): String = {
    val cleft = elems.map(_.split("\n"))

    if (cleft.exists(_.length != cleft.head.length)) throw new IllegalArgumentException()

    Array.fill[mutable.StringBuilder](cleft.head.length)(new mutable.StringBuilder())
      .zipWithIndex
      .map { case (acc, idx) =>
        acc
          .append(
            cleft
              .map(line => f"${line(idx)} ")
              .mkString
          )
          .append("\n")
      }
      .mkString
  }

  implicit class BoolTransform(val b: Boolean) extends AnyVal {

    def toInt: Int = if (b) 1 else 0

  }

  implicit class ByteTransform(val b: Byte) extends AnyVal {

    def toChunkedBinaryString: String = {
      val binaryString = b.toBinaryString

      val rs = if (binaryString.length == 8)
        binaryString
      else
        "0" * (8 - binaryString.length) + binaryString

      rs.grouped(4).reduce((acc, s) => acc + " " + s)
    }
  }

  implicit class IntTransform(val i: Int) extends AnyVal {

    def toBoolean: Boolean = if (i == 0) false else true

    def toChunkedBinaryString: String = {
      val binaryString = i.toBinaryString

      val rs =
        if (binaryString.length == 32)
          binaryString
        else
          "0" * (32 - binaryString.length) + binaryString

      rs.grouped(4).reduce((acc, s) => acc + " " + s)
    }

    @inline def <<|>>>(shift: Int): Int =
      if (shift < 0)
        i << -shift
      else
        i >>> shift

  }

}
