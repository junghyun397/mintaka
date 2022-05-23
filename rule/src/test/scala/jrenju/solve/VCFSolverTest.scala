package jrenju.solve

import jrenju.BoardIO.BoardToText
import jrenju.TestHelper.T2
import jrenju.notation.Pos
import jrenju.solve.SolutionMapper.SequenceToNode
import jrenju.solve.VCFSolver.VCFFinder
import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should
import utils.lang.Transform.joinHorizontal

class VCFSolverTest extends AnyFlatSpec with should.Matchers {

  def vcf(problem: String, answer: Int): Unit = {
    val board = problem.t2b

    val seq = board.findVCFSequence()

    val markedBoard = seq.foldLeft(board.clone()) { (board, idx) =>
      board.makeMove(idx)
    }

    println(joinHorizontal(board.boardText, markedBoard.boardText))
    println(f"${seq.length}, ${Pos.fromIdx(seq.last).toCartesian}, ${seq.toSolution}")

    seq.size should be (answer)
  }

  "VCF Sequences" should "analyze correctly" in {
    // BLACK 4-3 FORK
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . O . . . . 11
        |10 . . . . . . . . . X . X . . . 10
        | 9 . . . . . . . . . O . . . . . 9
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
      7
    )

    // BLACK 4-3 FORK
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . O . . . . . 11
        |10 . . . . . X . . . X . . . . . 10
        | 9 . . . . . O . O . . . O . . . 9
        | 8 . . O X X . . X . . X . . . . 8
        | 7 . . . . O . . . . X . . . . . 7
        | 6 . . . . . X O . . . . O . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . X . . . . . . . 4
        | 3 . . . . . . O . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      27
    )

    // BLACK 4-3 FORK
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . O . . . . . . . 11
        |10 . . . . . . . . . X O . . . . 10
        | 9 . . . . . . . O X . X . . . . 9
        | 8 . . . . . . . X O . X . O . . 8
        | 7 . . . . . . O . . O . X . . . 7
        | 6 . . . . . . . . . X . O O X . 6
        | 5 . . . . . . . . X O . X X . . 5
        | 4 . . . . . . . O . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      19
    )

    // BLACK 4-3 FORK
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . X . . . . . . . . 12
        |11 . . . . . . . . . . X . . . . 11
        |10 . . . . . . . O . . . . . . . 10
        | 9 . . . . O . . X . . O . . . . 9
        | 8 . . . . X O X X X . . . . . . 8
        | 7 . . . O . O O . . X . . . . . 7
        | 6 . . . . . . . . . . O . . . . 6
        | 5 . . . . . . . . . O X . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      19
    )

    // BLACK 4-3 FORK
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . . . . . . 11
        |10 . . . . . . . . . . . . . . . 10
        | 9 . . . . O . . . O . . . . . . 9
        | 8 . . . . . . . X O O . . . . . 8
        | 7 . . . . O X . X . . . . . . . 7
        | 6 . . . . . X . . O X . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . X . . . X . . . 4
        | 3 . . . . . . O . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      25
    )

    // BLACK 4-3 FORK
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 O O . . X . X . . O X O O X . 15
        |14 O . X . . O . . . . X . . . X 14
        |13 O . . . . . . . . . . . . . . 13
        |12 O . . . . X . . . . . . . . X 12
        |11 X . X . . . . . . . . . . . . 11
        |10 O . . O . . X . . . . . . X . 10
        | 9 O . . . . . . . O . . . . O X 9
        | 8 . . . O . . O X . . . . X . . 8
        | 7 . X . . . . . . O . . . X . . 7
        | 6 . . . . O . . . O . X . . . O 6
        | 5 . . . X . . . . . . . . . . X 5
        | 4 X O . . . X . . X . . X . X O 4
        | 3 . X . . . . . . . . . . . . O 3
        | 2 . O . . . . . . . . O . . X O 2
        | 1 X O . O . O . X . X O O X O . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      157
    )

    // BLACK 4-3 FORK
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 O . . . X . . . . . . . X . X 15
        |14 X . . . . O . . . O . . O . X 14
        |13 . . . . . . . O . . . . . O . 13
        |12 O . . . . . . . . . . X . . X 12
        |11 X . . . . . . . . . . . O . . 11
        |10 O . O . . . . . . . . . . . . 10
        | 9 O O X O . . . . X . . . O . . 9
        | 8 O . O O . . . X . O . . . . . 8
        | 7 . X . . . . . . . O . . X . . 7
        | 6 . . . . . . . . O . . . . . X 6
        | 5 X . . . . . . . . . . . X . X 5
        | 4 . . . . . . . . . . . . . X O 4
        | 3 X . . . . . . . . . . . . X . 3
        | 2 . . . . . . . X . . . . . . O 2
        | 1 X O O O . X . . X . X . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      173
    )

    // WHITE TRAP
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . O X . . O . . . . . . 13
        |12 . . . . . . . O . . . . . . . 12
        |11 . . . . O . X . . . . . . . . 11
        |10 . O . X . . O X O . . . . . . 10
        | 9 . . X X O . O X O . . . . . . 9
        | 8 . . . X X O . X X . . . . . . 8
        | 7 . . . . X . . . . . . . . . . 7
        | 6 . . . . . . . O X . . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      31
    )

    // WHITE TRAP
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . X . . . . . . . 15
        |14 . . . . . X O . . O . . . . . 14
        |13 . . . . X . O . . O . X . . . 13
        |12 . . . . . . O . X O . . . . . 12
        |11 . . . . O X X X O X O . . . . 11
        |10 . . . X O O O O X O X X . . . 10
        | 9 . . . . . . X O X . . O X . . 9
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
      19
    )

    // WHITE TRAP
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . O . . . O . . . 14
        |13 . . . . . . . X . . X . . . . 13
        |12 . . . . . . . . O . . . . . . 12
        |11 . . . O . X O X . . . . . . . 11
        |10 . . . X . . . . X . . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . . X . O X O . X . . . . 8
        | 7 . . . . . . O O X . . . . . . 7
        | 6 . . . . . . . . . . O . . . . 6
        | 5 . . . . . X . . . . X . O . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      25
    )
  }

}
