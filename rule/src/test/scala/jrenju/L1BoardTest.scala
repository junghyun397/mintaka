package jrenju

import jrenju.BoardIO.BoardToText
import jrenju.ComplexCase.T2
import jrenju.notation.{Flag, Pos}
import org.scalatest.flatspec._
import org.scalatest.matchers._

class L1BoardTest extends AnyFlatSpec with should.Matchers {

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
      .t2b(Pos.fromCartesian(2, "I").get.idx)

    for (_ <- 0 until 100000) {
      board.calculateGlobalL2Board()
    }
  }

  "board composer" should "work correctly" in {
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
      .t2b(Pos.fromCartesian(2, "I").get.idx)
      .calculateGlobalL2Board()
      .calculateL3Board()

    println(board.debugText)
  }

  "board slicer" should "work correctly" in {
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
      .t2b(Pos.fromCartesian(2, "I").get.idx)

    val strips = board.composeL2Strips()

    strips.foreach { strip => println(strip.forbidMask.map(Flag.flagToChar).mkString) }

    val globalStrips = board.composeGlobalL2Strips()

    globalStrips.foreach { strip => println(strip.forbidMask.map(Flag.flagToChar).mkString) }
  }

  "five-in-a-row board" should "has isEnd flag" in {
    val horizontal = (
      "   A B C D E F G H I J K L M N O   \n" +
      "15 . . . . . . . . . . . . . . . 15\n" +
      "14 . . . . . . . . . . . . . . . 14\n" +
      "13 . . . . . . . . . . . . . . . 13\n" +
      "12 . . . . . . . . . . . . . . . 12\n" +
      "11 . . . . . . . . . . . . . . . 11\n" +
      "10 . . . . . . . . . . . . . . . 10\n" +
      " 9 . . . . . . . . . . . . . . . 9 \n" +
      " 8 . . . . . O O O O O . . . . . 8 \n" +
      " 7 . . . . . . . . . . . . . . . 7 \n" +
      " 6 . . . . . . . . . . . . . . . 6 \n" +
      " 5 . . . . . . . . . . . . . . . 5 \n" +
      " 4 . . . . . . . . . . . . . . . 4 \n" +
      " 3 . . . . . . . . . . . . . . . 3 \n" +
      " 2 . . . . . . . . . . . . . . . 2 \n" +
      " 1 . . . . . . . . . . . . . . . 1 \n" +
      "   A B C D E F G H I J K L M N O   "
    ).t2b

    horizontal.calculateGlobalL2Board().winner should be (Flag.BLACK)

    val vertical = (
      "   A B C D E F G H I J K L M N O   \n" +
      "15 . . . . . . . . . . . . . . . 15\n" +
      "14 . . . . . . . . . . . . . . . 14\n" +
      "13 . . . . . . . . . . . . . . . 13\n" +
      "12 . . . . . . . . . . . . . . . 12\n" +
      "11 . . . . . . . . . . . . . . . 11\n" +
      "10 . . . . . . . O . . . . . . . 10\n" +
      " 9 . . . . . . . O . . . . . . . 9 \n" +
      " 8 . . . . . . . O . . . . . . . 8 \n" +
      " 7 . . . . . . . O . . . . . . . 7 \n" +
      " 6 . . . . . . . O . . . . . . . 6 \n" +
      " 5 . . . . . . . . . . . . . . . 5 \n" +
      " 4 . . . . . . . . . . . . . . . 4 \n" +
      " 3 . . . . . . . . . . . . . . . 3 \n" +
      " 2 . . . . . . . . . . . . . . . 2 \n" +
      " 1 . . . . . . . . . . . . . . . 1 \n" +
      "   A B C D E F G H I J K L M N O   "
    ).t2b

    vertical.calculateGlobalL2Board().winner should be (Flag.BLACK)

    val diagonalRightC1 = (
      "   A B C D E F G H I J K L M N O   \n" +
      "15 . . . . . . . . . . . . . . . 15\n" +
      "14 . . . . . . . . . . . . . . . 14\n" +
      "13 . . . . . . . . . . . . . . . 13\n" +
      "12 . . . . . . . . . . . . . . . 12\n" +
      "11 . . . . . . . . . . . . . . . 11\n" +
      "10 . . . . . . . . . . . . . . . 10\n" +
      " 9 . . . . . . . . O . . . . . . 9 \n" +
      " 8 . . . . . . . O . . . . . . . 8 \n" +
      " 7 . . . . . . O . . . . . . . . 7 \n" +
      " 6 . . . . . O . . . . . . . . . 6 \n" +
      " 5 . . . . O . . . . . . . . . . 5 \n" +
      " 4 . . . . . . . . . . . . . . . 4 \n" +
      " 3 . . . . . . . . . . . . . . . 3 \n" +
      " 2 . . . . . . . . . . . . . . . 2 \n" +
      " 1 . . . . . . . . . . . . . . . 1 \n" +
      "   A B C D E F G H I J K L M N O   "
    ).t2b

    diagonalRightC1.calculateGlobalL2Board().winner should be (Flag.BLACK)

    val diagonalRightC2 = (
        "   A B C D E F G H I J K L M N O   \n" +
        "15 . . . . O . . . . . . . . . . 15\n" +
        "14 . . . O . . . . . . . . . . . 14\n" +
        "13 . . O . . . . . . . . . . . . 13\n" +
        "12 . O . . . . . . . . . . . . . 12\n" +
        "11 O . . . . . . . . . . . . . . 11\n" +
        "10 . . . . . . . . . . . . . . . 10\n" +
        " 9 . . . . . . . . . . . . . . . 9 \n" +
        " 8 . . . . . . . . . . . . . . . 8 \n" +
        " 7 . . . . . . . . . . . . . . . 7 \n" +
        " 6 . . . . . . . . . . . . . . . 6 \n" +
        " 5 . . . . . . . . . . . . . . . 5 \n" +
        " 4 . . . . . . . . . . . . . . . 4 \n" +
        " 3 . . . . . . . . . . . . . . . 3 \n" +
        " 2 . . . . . . . . . . . . . . . 2 \n" +
        " 1 . . . . . . . . . . . . . . . 1 \n" +
        "   A B C D E F G H I J K L M N O   "
      ).t2b

    diagonalRightC2.calculateGlobalL2Board().winner should be (Flag.BLACK)

    val diagonalRightC3 = (
        "   A B C D E F G H I J K L M N O   \n" +
        "15 . . . . . . . . . . . . . . . 15\n" +
        "14 . . . . . . . . . . . . . . . 14\n" +
        "13 . . . . . . . . . . . . . . . 13\n" +
        "12 . . . . . . . . . . . . . . . 12\n" +
        "11 . . . . . . . . . . . . . . . 11\n" +
        "10 . . . . . . . . . . . . . . . 10\n" +
        " 9 . . . . . . . . . . . . . . . 9 \n" +
        " 8 . . . . . . . . . . . . . . . 8 \n" +
        " 7 . . . . . . . . . . . . . . . 7 \n" +
        " 6 . . . . . . . . . . . . . . . 6 \n" +
        " 5 . . . . . . . . . . . . . . O 5 \n" +
        " 4 . . . . . . . . . . . . . O . 4 \n" +
        " 3 . . . . . . . . . . . . O . . 3 \n" +
        " 2 . . . . . . . . . . . O . . . 2 \n" +
        " 1 . . . . . . . . . . O . . . . 1 \n" +
        "   A B C D E F G H I J K L M N O   "
      ).t2b

    diagonalRightC3.calculateGlobalL2Board().winner should be (Flag.BLACK)

    val diagonalLeftC1 = (
      "   A B C D E F G H I J K L M N O   \n" +
      "15 . . . . . . . . . . . . . . . 15\n" +
      "14 . . . . . . . . . . . . . . . 14\n" +
      "13 . . . . . . . . . . . . . . . 13\n" +
      "12 . . . . . . . . . . . . . . . 12\n" +
      "11 . . . . . . . . . . . . . . . 11\n" +
      "10 . . . . . O . . . . . . . . . 10\n" +
      " 9 . . . . . . O . . . . . . . . 9 \n" +
      " 8 . . . . . . . O . . . . . . . 8 \n" +
      " 7 . . . . . . . . O . . . . . . 7 \n" +
      " 6 . . . . . . . . . O . . . . . 6 \n" +
      " 5 . . . . . . . . . . . . . . . 5 \n" +
      " 4 . . . . . . . . . . . . . . . 4 \n" +
      " 3 . . . . . . . . . . . . . . . 3 \n" +
      " 2 . . . . . . . . . . . . . . . 2 \n" +
      " 1 . . . . . . . . . . . . . . . 1 \n" +
      "   A B C D E F G H I J K L M N O   "
    ).t2b

    diagonalLeftC1.calculateGlobalL2Board().winner should be (Flag.BLACK)

    val diagonalLeftC2 = (
        "   A B C D E F G H I J K L M N O   \n" +
        "15 . . . . . . . . . . . . . . . 15\n" +
        "14 . . . . . . . . . . . . . . . 14\n" +
        "13 . . . . . . . . . . . . . . . 13\n" +
        "12 . . . . . . . . . . . . . . . 12\n" +
        "11 . . . . . . . . . . . . . . . 11\n" +
        "10 . . . . . . . . . . . . . . . 10\n" +
        " 9 . . . . . . . . . . . . . . . 9 \n" +
        " 8 . . . . . . . . . . . . . . . 8 \n" +
        " 7 . . . . . . . . . . . . . . . 7 \n" +
        " 6 . . . . . . . . . . . . . . . 6 \n" +
        " 5 O . . . . . . . . . . . . . . 5 \n" +
        " 4 . O . . . . . . . . . . . . . 4 \n" +
        " 3 . . O . . . . . . . . . . . . 3 \n" +
        " 2 . . . O . . . . . . . . . . . 2 \n" +
        " 1 . . . . O . . . . . . . . . . 1 \n" +
        "   A B C D E F G H I J K L M N O   "
      ).t2b

    diagonalLeftC2.calculateGlobalL2Board().winner should be (Flag.BLACK)

    val diagonalLeftC3 = (
        "   A B C D E F G H I J K L M N O   \n" +
        "15 . . . . . . . . . . O . . . . 15\n" +
        "14 . . . . . . . . . . . O . . . 14\n" +
        "13 . . . . . . . . . . . . O . . 13\n" +
        "12 . . . . . . . . . . . . . O . 12\n" +
        "11 . . . . . . . . . . . . . . O 11\n" +
        "10 . . . . . . . . . . . . . . . 10\n" +
        " 9 . . . . . . . . . . . . . . . 9 \n" +
        " 8 . . . . . . . . . . . . . . . 8 \n" +
        " 7 . . . . . . . . . . . . . . . 7 \n" +
        " 6 . . . . . . . . . . . . . . . 6 \n" +
        " 5 . . . . . . . . . . . . . . . 5 \n" +
        " 4 . . . . . . . . . . . . . . . 4 \n" +
        " 3 . . . . . . . . . . . . . . . 3 \n" +
        " 2 . . . . . . . . . . . . . . . 2 \n" +
        " 1 . . . . . . . . . . . . . . . 1 \n" +
        "   A B C D E F G H I J K L M N O   "
      ).t2b

    diagonalLeftC3.calculateGlobalL2Board().winner should be (Flag.BLACK)

  }

  "3-attack points" should "increases attack point" in {}

  "4-attack points" should "increases attack point" in {}

  "5-in-a-row board" should "has end flag" in {}

}
