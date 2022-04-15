package utils

import scala.language.implicitConversions

package object lang {

  implicit class StringArrayTransform(val value: Array[String]) {

    def mergeHorizontal: String = {
      val cleft = value.map(_.split("\n"))
      Array.fill[StringBuilder](cleft(0).length)(new StringBuilder())
        .zipWithIndex
        .map(builderIdx =>
          builderIdx._1
            .append(
              cleft
                .map(line => f"${line(builderIdx._2)} ")
                .mkString
            )
            .append("\n")
        )
        .mkString
    }
    
  }

  implicit class IntTransform(val value: Int) {

    def dotIfZero: String = if (value == 0) "." else value.toString

  }

  implicit class BoolTransform(val value: Boolean) extends AnyVal {

    def toInt: Int = if (value) 1 else 0

  }

}
