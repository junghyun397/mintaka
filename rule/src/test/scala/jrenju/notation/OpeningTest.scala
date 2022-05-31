package jrenju.notation

import jrenju.TestHelper.T2
import jrenju.solve.LargeMoveGenerator
import org.scalatest.flatspec._
import org.scalatest.matchers._

class OpeningTest extends AnyFlatSpec with should.Matchers {

  def moveGenerator(problem: String): Unit = {
    println(LargeMoveGenerator.collectValidMoves(problem.t2b).mkString("Array(", ", ", ")"))
  }

  "d" should "a" in {
    moveGenerator(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . . . . . . 11
        |10 . . . . . . . . . . . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . . . . . X . . . . . . . 8
        | 7 . . . . . . . . . . . . . . . 7
        | 6 . . . . . . . . . . . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
    )
  }

}
