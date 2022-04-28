package jrenju.solve

import jrenju.TestHelper.T2
import jrenju.protocol.SolutionNode.SequenceToNode
import jrenju.solve.VCFSolver.VCFFinder
import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should

class VCFSolverTest extends AnyFlatSpec with should.Matchers {

  def vcf(problem: String, answer: Boolean): Unit = {
    val board = problem.t2b
      .calculateGlobalPoints()
      .calculateForbids()

    val solution = board.findVCFSequence()
      .toSolution
      .toString

    println(problem)
    println(solution)

    solution.nonEmpty should be (answer)
  }

  "VCF Points" should "analyze correctly" in {
    // BLACK 4-3 FORK
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . X . . . . 11
        |10 . . . . . . . . . O . O . . . 10
        | 9 . . . . . . . . . X . . . . . 9
        | 8 . . . . . . . O . O O X . . . 8
        | 7 . . . . . . O . O X . . . . . 7
        | 6 . . . . . . . X X . . . . . . 6
        | 5 . . . . . . X . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      answer = true
    )

    // BLACK 4-3 FORK
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . X . . . . . 11
        |10 . . . . . O . . . O . . . . . 10
        | 9 . . . . . X . X . . . X . . . 9
        | 8 . . X O O . . O . . O . . . . 8
        | 7 . . . . X . . . . O . . . . . 7
        | 6 . . . . . O X . . . . X . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . O . . . . . . . 4
        | 3 . . . . . . X . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      answer = true
    )

    // BLACK 4-3 FORK
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . X . . . . . . . 11
        |10 . . . . . . . . . O X . . . . 10
        | 9 . . . . . . . X O . O . . . . 9
        | 8 . . . . . . . O X . O . X . . 8
        | 7 . . . . . . X . . X . O . . . 7
        | 6 . . . . . . . . . O . X X O . 6
        | 5 . . . . . . . . O X . O O . . 5
        | 4 . . . . . . . X . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      answer = true
    )

    // BLACK 4-3 FORK
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . O . . . . . . . . 12
        |11 . . . . . . . . . . O . . . . 11
        |10 . . . . . . . X . . . . . . . 10
        | 9 . . . . X . . O . . X . . . . 9
        | 8 . . . . O X O O O . . . . . . 8
        | 7 . . . X . X X . . O . . . . . 7
        | 6 . . . . . . . . . . X . . . . 6
        | 5 . . . . . . . . . X O . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      answer = true
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
        | 9 . . . . X . . . X . . . . . . 9
        | 8 . . . . . . . O X X . . . . . 8
        | 7 . . . . X O . O . . . . . . . 7
        | 6 . . . . . O . . X O . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . O . . . O . . . 4
        | 3 . . . . . . X . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      answer = true
    )

    // WHITE TRAP
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . X O . . X . . . . . . 13
        |12 . . . . . . . X . . . . . . . 12
        |11 . . . . X . O . . . . . . . . 11
        |10 . X . O . . X O X . . . . . . 10
        | 9 . . O O X . X O X . . . . . . 9
        | 8 . . . O O X . O O . . . . . . 8
        | 7 . . . . O . . . . . . . . . . 7
        | 6 . . . . . . . X O . . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      answer = true
    )

    // WHITE TRAP
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . O . . . . . . . 15
        |14 . . . . . O X . . X . . . . . 14
        |13 . . . . O . X . . X . O . . . 13
        |12 . . . . . . X . O X . . . . . 12
        |11 . . . . X O O O X O X . . . . 11
        |10 . . . O X X X X O X O O . . . 10
        | 9 . . . . . . O X O . . X O . . 9
        | 8 . . . . . . . O . . . . . . . 8
        | 7 . . . . . . . . . . . . . . . 7
        | 6 . . . . . . . . . . . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      answer = true
    )

    // WHITE TRAP
    vcf(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . X . . . X . . . 14
        |13 . . . . . . . O . . O . . . . 13
        |12 . . . . . . . . X . . . . . . 12
        |11 . . . X . O X O . . . . . . . 11
        |10 . . . O . . . . O . . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . . O . X O X . O . . . . 8
        | 7 . . . . . . X X O . . . . . . 7
        | 6 . . . . . . . . . . X . . . . 6
        | 5 . . . . . O . . . . O . X . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      answer = true
    )
  }

}
