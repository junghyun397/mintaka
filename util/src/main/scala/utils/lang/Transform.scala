package utils.lang

object Transform {
  
  implicit class StringArrayTransform(val xs: Array[String]) {

    def mergeHorizontal: String = {
      val cleft = xs.map(_.split("\n"))
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

  implicit class BoolTransform(val b: Boolean) {
    
    def toInt: Int = if (b) 1 else 0
    
  }
  
  implicit class IntTransform(val i: Int) {
    
    def toBoolean: Boolean = if (i == 0) false else true
    
  }
  
}
