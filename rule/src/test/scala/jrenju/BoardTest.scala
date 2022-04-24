package jrenju

import jrenju.TestHelper.T2
import jrenju.notation.Pos
import org.scalatest.flatspec._
import org.scalatest.matchers._

class BoardTest extends AnyFlatSpec with should.Matchers {

  "stress test" should "work correctly" in {
    val board = (
      "   A B C D E F G H I J K L M N O   \n" +
        "15 . . . . . . . . . . . . . . . 15\n" +
        "14 . . . . . . . . . . . . . . . 14\n" +
        "13 . . . . . . . . . O . . . . . 13\n" +
        "12 . . . . . . . O X X . . . . . 12\n" +
        "11 . . . . . . . . . . O O . . . 11\n" +
        "10 . . . . O X O O O X . . . . . 10\n" +
        " 9 . . . X X O O X X . . . . . . 9 \n" +
        " 8 . . . O X X X O X O . . . . . 8 \n" +
        " 7 . . . X . O O O X X O . . . . 7 \n" +
        " 6 . . O O X X X . O O X . . . . 6 \n" +
        " 5 . X O O O X . X O X . O . . . 5 \n" +
        " 4 . . O X X X O X O X . . . . . 4 \n" +
        " 3 . . . . . O . . O . . . . . . 3 \n" +
        " 2 . . . . . . . . X . . . . . . 2 \n" +
        " 1 . . . . . . . . . . . . . . . 1 \n" +
        "   A B C D E F G H I J K L M N O   "
      )
      .t2b

    for (_ <- 0 until 100_000) {
      board
        .makeMove(Pos.fromCartesian("c", 7).get)
        .calculateGlobalL2Board()
        .calculateL3Board()
        .calculateDeepL3Board()
    }
  }

}
