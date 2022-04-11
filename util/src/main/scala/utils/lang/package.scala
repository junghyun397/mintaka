package utils

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

  implicit class ByteTransform(val value: Byte) {

    def dotIfZero: String = if (value == 0) "." else value.toString

  }

}
