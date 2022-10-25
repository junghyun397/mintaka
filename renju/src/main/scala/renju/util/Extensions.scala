package renju.util

import java.util.concurrent.ConcurrentHashMap
import scala.collection.mutable

object Extensions {

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

  implicit class StringExtensions(val s: String) extends AnyVal {

    def trimLines: String = {
      val lines = for {
        line <- s.split("\n")
        trim = line.reverse.dropWhile(_ == ' ').reverse
      } yield trim

      lines.mkString("\n")
    }

  }

  implicit class BoolExtensions(val b: Boolean) extends AnyVal {

    def toInt: Int = if (b) 1 else 0

  }

  implicit class ByteExtensions(val b: Byte) extends AnyVal {

    def toChunkedBinaryString: String = {
      val binaryString = b.toBinaryString

      val rs = if (binaryString.length == 8)
        binaryString
      else
        "0" * (8 - binaryString.length) + binaryString

      rs.grouped(4).reduce((acc, s) => acc + " " + s)
    }
  }

  implicit class IntExtensions(val i: Int) extends AnyVal {

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

    @inline def shi(shift: Int): Int =
      if (shift < 0)
        i << -shift
      else
        i >>> shift

  }

  implicit class ConcurrentMapExtension[K, V](private val xm: ConcurrentHashMap[K, V]) extends AnyVal {

    def getOrElseUpdate(key: K, f: () => V): V = {
      var value = xm.get(key)
      if (value == null) {
        value = f()
        xm.put(key, value)
      }
      value
    }

  }

}
