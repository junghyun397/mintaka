package jrenju

import jrenju.BoardIO.BoardToText
import jrenju.ComplexCase.T2
import org.scalatest._
import org.scalatest.flatspec._
import org.scalatest.matchers._

class L2BoardTest extends AnyFlatSpec with should.Matchers {

  "Double-3 forbidden points" should "has forbidden flag" in {
    val case1 = (
      "   A B C D E F G H I J K L M N O   \n" +
      "15 . . . . . . . . . . . . . . X 15\n" +
      "14 . . . . . . . . . . . . . . . 14\n" +
      "13 . . . . . . . . . . . . . . . 13\n" +
      "12 . . . . . . . . . . . . . . . 12\n" +
      "11 . . . . O . O . . . . . . . . 11\n" +
      "10 . . . . . . . . . O . . . . . 10\n" +
      " 9 . . . . . . . . . . . . . . . 9 \n" +
      " 8 . . . . . . O O . . . . . . . 8 \n" +
      " 7 . . . . O . . . O O . . . . . 7 \n" +
      " 6 . . . . O . O . O . . . . . . 6 \n" +
      " 5 . . . . . X . . . O O O . . . 5 \n" +
      " 4 . . . . X . . . O . . . . . . 4 \n" +
      " 3 . . . . . . . . . . . . . . . 3 \n" +
      " 2 . . . . . . . . . . . . . . . 2 \n" +
      " 1 . . . . . . . . . . . . . . . 1 \n" +
      "   A B C D E F G H I J K L M N O   "
    ).t2b
      .calculateGlobalL2Board()
      .calculateL3Board()

    println(case1.debugText)

  }

  "Double-4 forbidden points" should "has forbidden flag" in {}

  "Over-6 forbidden points" should "has forbidden flag" in {}

}
