package renju

import org.scalatest.flatspec._
import org.scalatest.matchers._
import renju.BoardIO.BoardIOExtension
import renju.ScalaBoard.structOps
import renju.TestHelper.S2
import renju.notation.Pos
import renju.util.Extensions.StringExtensions

class StructOpsTest extends AnyFlatSpec with should.Matchers {

  def deepForbid(problem: String, answer: String): Unit = {
    val board = problem.s2b

    answer should include (board.boardString(false).trimLines)
  }

  "StructOps" should "handle edge cases" in {
    deepForbid(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . . . . . . 11
        |10 . . . . . . . . . . . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . . . O . X . . . . . . . 8
        | 7 . . . . O . X O O . . . . . . 7
        | 6 . . . . . O O X . . . . . . . 6
        | 5 . . . . . X X O X . . . . . . 5
        | 4 . . . . . . X . X X . . . . . 4
        | 3 . . . . . . . . . . O . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . . . . . . 11
        |10 . . . . . . . . . . . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . . . O . X . . . . . . . 8
        | 7 . . . . O . X O O . . . . . . 7
        | 6 . . . . . O O X . . . . . . . 6
        | 5 . . . . . X X O X . . . . . . 5
        | 4 . . . . . . X . X X . . . . . 4
        | 3 . . . . . . . . . . O . . . . 3
        | 2 . . . . . . . . 3 . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
    )
  }

  "StructOps" should "resolve basic forbidden points" in {
    deepForbid(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . X . . . . . . . . . . 13
        |12 . . X . X . . . . . . . . . . 12
        |11 . X . . X . . . . . . . . . . 11
        |10 X . . . O . . . . . . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . X X . X X X . . . . . . 8
        | 7 . . . . . . . . . . . . . . . 7
        | 6 . . . . . . . . . . . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . X . . . . . . . . . X X . . 4
        | 3 . X . X . . . . . . X . X . . 3
        | 2 . . . X . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . 4 . . . . . . . . . . 14
        |13 . . . . X . . . . . . . . . . 13
        |12 . . X . X . . . . . . . . . . 12
        |11 . X . . X . . . . . . . . . . 11
        |10 X . . . O . . . . . . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . X X 6 X X X . . . . . . 8
        | 7 . . . . . . . . . . . . . . . 7
        | 6 . . . . . . . . . . . . . . . 6
        | 5 . . . . . . . . . . . . 3 . . 5
        | 4 . X . . . . . . . . . X X . . 4
        | 3 . X . X . . . . . . X . X . . 3
        | 2 . . . X . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
    )
  }

  "StructOps" should "resolve pseudo forbidden points" in {
    deepForbid(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . O . . . . . . . 11
        |10 . . . . . O . . . . . . . . . 10
        | 9 . . . . . X X . . . . . . . . 9
        | 8 . . . . . . O X O . . . . . . 8
        | 7 . . . . . . O X O . . . . . . 7
        | 6 . . . . . . . . X X X O . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . O . . . . . . . 11
        |10 . . . . . O . . . . . . . . . 10
        | 9 . . . . . X X . . . . . . . . 9
        | 8 . . . . . . O X O . . . . . . 8
        | 7 . . . . . . O X O . . . . . . 7
        | 6 . . . . . . . . X X X O . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin
    )

    deepForbid(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . O . . . . . . . . . . 11
        |10 . . . . . X . . . . . . . . . 10
        | 9 . . . . . . X O O X . . . . . 9
        | 8 . . . . . . . X X . . . . . . 8
        | 7 . . . . . X . O O . . . . . . 7
        | 6 . . . . O X X . O X . . . . . 6
        | 5 . . . . . O . O . X . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . O . . . . . . . . . . 11
        |10 . . . . . X . . . . . . . . . 10
        | 9 . . . . . . X O O X . . . . . 9
        | 8 . . . . . . . X X . . . . . . 8
        | 7 . . . . . X . O O . . . . . . 7
        | 6 . . . . O X X . O X . . . . . 6
        | 5 . . . . . O . O . X . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin
    )

    deepForbid(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . O . . . . . . . . . . 12
        |11 . . . . . O . . . . . . . . . 11
        |10 . . . . . X O . O X . . . . . 10
        | 9 . . . . . . X . X . . . . . . 9
        | 8 . . . . . . . X O X . . . . . 8
        | 7 . . . . . X O X O X O . . . . 7
        | 6 . . . . . X . . . O . . . . . 6
        | 5 . . . . . O . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . O . . . . . . . . . . 12
        |11 . . . . . O . . . . . . . . . 11
        |10 . . . . . X O . O X . . . . . 10
        | 9 . . . . . . X . X . . . . . . 9
        | 8 . . . . . . . X O X . . . . . 8
        | 7 . . . . . X O X O X O . . . . 7
        | 6 . . . . . X . . . O . . . . . 6
        | 5 . . . . . O . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
       """.stripMargin,
    )

    deepForbid(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . O . . . . . . . . . 12
        |11 . . . . . X O . O . . . . . . 11
        |10 . . . . . . X . X . O . . . . 10
        | 9 . . . . O . O X . . . . . . . 9
        | 8 . . . . . X O X O . . . . . . 8
        | 7 . . . . . X . . X . . . . . . 7
        | 6 . . . . . . . . . . . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . O . . . . . . . . . 12
        |11 . . . . . X O . O . . . . . . 11
        |10 . . . . . . X . X . O . . . . 10
        | 9 . . . . O . O X . . . . . . . 9
        | 8 . . . . . X O X O . . . . . . 8
        | 7 . . . . . X . 3 X . . . . . . 7
        | 6 . . . . . . . . . . . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
    )

    deepForbid(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . O . . O . . . . . . . . 11
        |10 . . . . X . . X . O . . . . . 10
        | 9 . . . . O X O X X . . . . . . 9
        | 8 . . . . . . X X . . . . . . . 8
        | 7 . . . . . . O O X X . . . . . 7
        | 6 . . . . . X . . . . . . . . . 6
        | 5 . . . . O X . . . . . . . . . 5
        | 4 . . . . . O . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . O . . O . . . . . . . . 11
        |10 . . . . X . . X . O . . . . . 10
        | 9 . . . . O X O X X . . . . . . 9
        | 8 . . . . . . X X 3 . . . . . . 8
        | 7 . . . . . . O O X X . . . . . 7
        | 6 . . . . . X . . . . . . . . . 6
        | 5 . . . . O X . . . . . . . . . 5
        | 4 . . . . . O . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
    )

    deepForbid(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . O . . O . . . . . . . . 11
        |10 . . . . X . . X . O . . . . . 10
        | 9 . . . . O X O X X . . . . . . 9
        | 8 . . . . . . X X . . O . . . . 8
        | 7 . . . . . . O O X X . . . . . 7
        | 6 . . . . . X O . . . . . . . . 6
        | 5 . . . . O X . . . . . . . . . 5
        | 4 . . . . . O . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . O . . O . . . . . . . . 11
        |10 . . . . X . 3 X 3 O . . . . . 10
        | 9 . . . . O X O X X . . . . . . 9
        | 8 . . . . . . X X 3 . O . . . . 8
        | 7 . . . . . . O O X X . . . . . 7
        | 6 . . . . . X O . . . . . . . . 6
        | 5 . . . . O X . . . . . . . . . 5
        | 4 . . . . . O . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
    )

    deepForbid(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . O . . . . . . . 12
        |11 . . . . . . . X . . . . . . . 11
        |10 . . . . . O . X . . . . . . . 10
        | 9 . . . . . X O X O . . . . . . 9
        | 8 . . . . . X . X . O . . . . . 8
        | 7 . . O X X X X O X . . . . . . 7
        | 6 . . . . . X . O . O . . . . . 6
        | 5 . . . . X O O . . . . . . . . 5
        | 4 . . . O . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . O . . . . . . . 12
        |11 . . . . . . . X . . . . . . . 11
        |10 . . . . . O . X . . . . . . . 10
        | 9 . . . . . X O X O . . . . . . 9
        | 8 . . . . 3 X . X . O . . . . . 8
        | 7 . . O X X X X O X . . . . . . 7
        | 6 . . . . 3 X . O . O . . . . . 6
        | 5 . . . . X O O . . . . . . . . 5
        | 4 . . . O . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
    )

  }

  "StructOps" should "handle uncommon forbidden points" in {
    deepForbid(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . O 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . X . X . . . . . . . . 11
        |10 . . . . . . . . . X . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . . . . X X . . . . . . . 8
        | 7 . . . . X . . . X X . . . . . 7
        | 6 . . . . X . X . X . . . . . . 6
        | 5 . . . . . . . . . X X X . . . 5
        | 4 . . . . . . . . X . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . O 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . X . X . . . . . . . . 11
        |10 . . . . . . . . . X . . . . . 10
        | 9 . . . . . . 4 . . . . . . . . 9
        | 8 . . . . 3 . X X 3 . . . . . . 8
        | 7 . . . . X . 3 . X X . 3 . . . 7
        | 6 . . . . X . X . X . 3 . . . . 6
        | 5 . . . . . . . 3 4 X X X . . . 5
        | 4 . . . . . . . . X . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
    )
  }

  "StructOps" should "handle recursive forbidden points" in {
    deepForbid(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . . . . . . 11
        |10 . . . . . . O O . . . . . . . 10
        | 9 . . . . . . X X . . . . . . . 9
        | 8 . . . . O X X X X O . . . . . 8
        | 7 . . . . O X X X X O . . . . . 7
        | 6 . . . . . . X X . . . . . . . 6
        | 5 . . . . . . O O . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . . . . . . 11
        |10 . . . . . . O O . . . . . . . 10
        | 9 . . . . . 3 X X 3 . . . . . . 9
        | 8 . . . . O X X X X O . . . . . 8
        | 7 . . . . O X X X X O . . . . . 7
        | 6 . . . . . 3 X X 3 . . . . . . 6
        | 5 . . . . . . O O . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
    )
  }

  def trap(problem: String, answer: Set[String]): Unit = {
    val board = problem.s2b

    val (threeSide, fourSide) = board.collectTrapPoints()

    (threeSide ++ fourSide).map(Pos.fromIdx).map(_.toString).toSet should be (answer)
  }

  "StructOps" should "resolve trap moves" in {
    trap(
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
        | 7 . . . . . . . X . . . . . . . 7
        | 6 . . . . . . . . X X . . . . . 6
        | 5 . . . . . . O . . . . . . . . 5
        | 4 . . . . . O . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      Set("i7", "e3")
    )

    trap(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . . . . . . 11
        |10 . . . . . . . . . . . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . . . . . X . X . . . . . 8
        | 7 . . . . . . . X O . . . . . . 7
        | 6 . . . . . . . . X X . . . . . 6
        | 5 . . . . . . O . . . . . . . . 5
        | 4 . . . . . O . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      Set("e3")
    )

    trap(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . . . . . . 11
        |10 . . . . . . . . . . . . . . . 10
        | 9 . . . . . . . . . X . . . . . 9
        | 8 . . . . . . . X . X . . . . . 8
        | 7 . . . . . . . . . . . . . . . 7
        | 6 . . . . X O O . O . . . . . . 6
        | 5 . . . . . . . . . . X . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      Set("h6")
    )
  }

}
