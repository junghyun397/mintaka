package jrenju

import jrenju.ComplexCase.T2b
import org.scalatest.flatspec._
import org.scalatest.matchers._

class L1BoardTest extends AnyFlatSpec with should.Matchers {

  "five-in-a-row board" should "has isEnd flag" in {
    val horizontal = (
      "   A B C D E F G H I J K L M N O   \n" +
      "15 . . . . . . . . . . . . . . . 15\n" +
      "14 . . . . . . . . . . . . . . . 14\n" +
      "13 . . . . . . . . . . . . . . . 13\n" +
      "12 . . . . . . . . . . . . . . . 12\n" +
      "11 . . . . . . . . . . . . . . . 11\n" +
      "10 . . . . . . . . . . . . . . . 10\n" +
      " 9 . . . . . . . . . . . . . . .  9\n" +
      " 8 . . . . . O O O O O . . . . .  8\n" +
      " 7 . . . . . . . . . . . . . . .  7\n" +
      " 6 . . . . . . . . . . . . . . .  6\n" +
      " 5 . . . . . . . . . . . . . . .  5\n" +
      " 4 . . . . . . . . . . . . . . .  4\n" +
      " 3 . . . . . . . . . . . . . . .  3\n" +
      " 2 . . . . . . . . . . . . . . .  2\n" +
      " 1 . . . . . . . . . . . . . . .  1\n" +
      "   A B C D E F G H I J K L M N O   "
    ).t2b

    horizontal.calculateL2Board().isEnd should be (true)

    val vertical = (
      "   A B C D E F G H I J K L M N O   \n" +
      "15 . . . . . . . . . . . . . . . 15\n" +
      "14 . . . . . . . . . . . . . . . 14\n" +
      "13 . . . . . . . . . . . . . . . 13\n" +
      "12 . . . . . . . . . . . . . . . 12\n" +
      "11 . . . . . . . . . . . . . . . 11\n" +
      "10 . . . . . . . O . . . . . . . 10\n" +
      " 9 . . . . . . . O . . . . . . .  9\n" +
      " 8 . . . . . . . O . . . . . . .  8\n" +
      " 7 . . . . . . . O . . . . . . .  7\n" +
      " 6 . . . . . . . O . . . . . . .  6\n" +
      " 5 . . . . . . . . . . . . . . .  5\n" +
      " 4 . . . . . . . . . . . . . . .  4\n" +
      " 3 . . . . . . . . . . . . . . .  3\n" +
      " 2 . . . . . . . . . . . . . . .  2\n" +
      " 1 . . . . . . . . . . . . . . .  1\n" +
      "   A B C D E F G H I J K L M N O   "
    ).t2b

    vertical.calculateL2Board().isEnd should be (true)

    val diagonalRight = (
      "   A B C D E F G H I J K L M N O   \n" +
      "15 . . . . . . . . . . . . . . . 15\n" +
      "14 . . . . . . . . . . . . . . . 14\n" +
      "13 . . . . . . . . . . . . . . . 13\n" +
      "12 . . . . . . . . . . . . . . . 12\n" +
      "11 . . . . . . . . . . . . . . . 11\n" +
      "10 . . . . . . . . . . . . . . . 10\n" +
      " 9 . . . . . . . . O . . . . . .  9\n" +
      " 8 . . . . . . . O . . . . . . .  8\n" +
      " 7 . . . . . . O . . . . . . . .  7\n" +
      " 6 . . . . . O . . . . . . . . .  6\n" +
      " 5 . . . . O . . . . . . . . . .  5\n" +
      " 4 . . . . . . . . . . . . . . .  4\n" +
      " 3 . . . . . . . . . . . . . . .  3\n" +
      " 2 . . . . . . . . . . . . . . .  2\n" +
      " 1 . . . . . . . . . . . . . . .  1\n" +
      "   A B C D E F G H I J K L M N O   "
    ).t2b

    diagonalRight.calculateL2Board().isEnd should be (true)

    val diagonalLeft = (
      "   A B C D E F G H I J K L M N O   \n" +
      "15 . . . . . . . . . . . . . . . 15\n" +
      "14 . . . . . . . . . . . . . . . 14\n" +
      "13 . . . . . . . . . . . . . . . 13\n" +
      "12 . . . . . . . . . . . . . . . 12\n" +
      "11 . . . . . . . . . . . . . . . 11\n" +
      "10 . . . . . O . . . . . . . . . 10\n" +
      " 9 . . . . . . O . . . . . . . .  9\n" +
      " 8 . . . . . . . O . . . . . . .  8\n" +
      " 7 . . . . . . . . O . . . . . .  7\n" +
      " 6 . . . . . . . . . O . . . . .  6\n" +
      " 5 . . . . . . . . . . . . . . .  5\n" +
      " 4 . . . . . . . . . . . . . . .  4\n" +
      " 3 . . . . . . . . . . . . . . .  3\n" +
      " 2 . . . . . . . . . . . . . . .  2\n" +
      " 1 . . . . . . . . . . . . . . .  1\n" +
      "   A B C D E F G H I J K L M N O   "
    ).t2b

    diagonalLeft.calculateL2Board().isEnd should be (true)
  }

  "3-attack points" should "increases attack point" in {}

  "4-attack points" should "increases attack point" in {}

  "5-in-a-row board" should "has end flag" in {}

}
