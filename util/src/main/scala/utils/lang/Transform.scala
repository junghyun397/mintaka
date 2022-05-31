package utils.lang

object Transform {
  
  def joinHorizontal(elems: String*): String = {
    val cleft = elems.map(_.split("\n"))
    if (cleft.exists(_.length != cleft.head.length)) throw new IllegalArgumentException()
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

  implicit class BoolTransform(val b: Boolean) {
    
    def toInt: Int = if (b) 1 else 0
    
  }

  implicit class ByteTransform(val b: Byte) {

    def toGroupedBinaryString: String = {
      val binaryString = b.toBinaryString

      val rs = if (binaryString.length == 8)
        binaryString
      else
        "0" * (8 - binaryString.length) + binaryString

      rs.grouped(4).reduce((acc, s) => acc + " " + s)
    }
  }
  
  implicit class IntTransform(val i: Int) {
    
    def toBoolean: Boolean = if (i == 0) false else true

    def toGroupedBinaryString: String = {
      val binaryString = i.toBinaryString

      val rs = if (binaryString.length == 32)
        binaryString
      else
        "0" * (32 - binaryString.length) + binaryString
        
      rs.grouped(4).reduce((acc, s) => acc + " " + s)
    }
    
  }

}
