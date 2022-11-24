package engine.move

import org.scalatest.flatspec.*
import org.scalatest.matchers.*
import renju.BoardIO
import renju.BoardIO.BoardIOExtension
import renju.TestHelper.S2
import renju.notation.Renju
import renju.util.Extensions.StringExtensions

class ThreatMoveGeneratorTest extends AnyFlatSpec with should.Matchers {

  def validMove(problem: String, answer: String): Unit = {
    val board = problem.s2b

    val moves = ThreatMoveGenerator.collectValidMoves(board)

    val marks = for {
      idx <- 0 until Renju.BOARD_SIZE
      mark = moves.contains(idx)
    } yield mark

    val marked = board.attributeString(false)(_ => marks.toArray)(if (_) "O" else ".")

    answer should include (marked.trimLines)
  }

  "ThreatMoveGenerator" should "resolve threat moves" in {
    validMove(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . O . . . . 11
        |10 . . . . . . . . . X . X . . . 10
        | 9 . . . . . . . . X O . . . . . 9
        | 8 . . . . . . . X . X X O . . . 8
        | 7 . . . . . . X . X O . . . . . 7
        | 6 . . . . . . . O O . . . . . . 6
        | 5 . . . . . . O . . . . . . . . 5
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
        |10 . . . . . . . . . . . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . . . . . . . . . . . . . 8
        | 7 . . . . . . . . . . . . . . . 7
        | 6 . . . . . O . . . . . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin
    )
  }

}
