package jrenju

import jrenju.TestHelper.T2
import jrenju.solve.VCFSolver.VCFFinder
import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should

class Benchmark extends AnyFlatSpec with should.Matchers {

  "Complex VCF benchmark" should "run rapidly" in {
    val case1 = """
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
    """.t2b
      .calculateGlobalPoints()
      .calculateForbids()

    val case2 = """
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
    """.t2b
      .calculateGlobalPoints()
      .calculateForbids()

    for (_ <- 1 to 1000) {
      case1.findVCFSequence()
      case2.findVCFSequence()
    }
  }

}
