package jrenju.solve

import jrenju.TestHelper.T2
import jrenju.protocol.SolutionNode.SequenceToNode
import jrenju.solve.VCFSolver.VCFFinder
import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should

class VCFSolverTest extends AnyFlatSpec with should.Matchers {

  def vcf(problem: String, answer: Boolean): Unit = {
    val board = problem.t2b

    val seq = board.findVCFSequence()

    println(problem)
    println(seq.length)
    println(seq.toSolution)

    seq.toString.nonEmpty should be (answer)
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
      answer = true
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
      answer = true
    )

    // BLACK 4-3 FORK
    // "h8,g8,c14,b15,e15,f14,g15,a15,a11,a12,c11,a10,b7,a9,a1,a13,b3,a14,a4,b4,d5,e6,f4,d8,i4,d10,k6,i6,l4,i7,n4,i9,o5,o4,m7,o6,m8,n9,n10,o3,n2,o2,m1,n1,j1,k1,h1,l1,o14,k2,o12,b2,n15,m15,k15,l15,k14,j15,f12,b1,g10,d1,o9,f1"
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
      answer = true
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
      answer = true
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
      answer = true
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
      answer = true
    )
  }

}
