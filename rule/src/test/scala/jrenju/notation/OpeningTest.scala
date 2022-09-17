package jrenju.notation

import jrenju.BoardIO.BoardToText
import jrenju.TestHelper.T2
import jrenju.solve.LargeMoveGenerator
import org.scalatest.flatspec._
import org.scalatest.matchers._

class OpeningTest extends AnyFlatSpec with should.Matchers {

  def moveGenerator(problem: String): Unit = {
    val moves = LargeMoveGenerator.collectValidMoves(problem.t2b)
    val ids = Array.fill(Renju.BOARD_SIZE)(false).zipWithIndex.map(combined => moves.contains(combined._2))
    println(problem.t2b.map(_.color).size)
    println(problem.t2b.attributeText(markLastMove = false)(_ => ids)(if (_) "@" else "."))
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
        |10 . . . . . . . . X . . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . . . . O X . . . . . . . 8
        | 7 . . . . . O . X . . . . . . . 7
        | 6 . . . . O . . X . . . . . . . 6
        | 5 . . . O . . . X . . . . . . . 5
        | 4 . . . . . . . . . . . . O . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
    )
  }

}
