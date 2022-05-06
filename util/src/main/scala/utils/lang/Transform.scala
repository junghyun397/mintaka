package utils.lang

object Transform {
  
  def joinHorizontal(elems: String*): String = {
    val cleft = elems.map(_.split("\n"))
    if (cleft.exists(_.length != cleft.head.length)) throw new Exception()
    Array.fill[StringBuilder](cleft.head.length)(new StringBuilder())
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

  implicit class BoolTransform(val b: Boolean) extends AnyVal {
    
    def toInt: Int = if (b) 1 else 0
    
  }
  
  implicit class IntTransform(val i: Int) extends AnyVal {
    
    def toBoolean: Boolean = if (i == 0) false else true
    
  }
  
}
