package engine.vcf

import engine.cache.LRUMemo
import engine.search.vcf.VCFSolver
import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should
import renju.TestHelper.T2
import renju.protocol.Solution

import scala.scalajs.js.JSON

class Benchmark extends AnyFlatSpec with should.Matchers {

  // 1155 ms
  "complex VCF benchmark" should "run rapidly" in {
    val memoBlack = new LRUMemo()
    val black43Fork = """
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
    """.t2b

    val memoWhite = new LRUMemo()
    val whiteTrap = """
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
    """.t2b

    for (_ <- 1 to 2) {
      VCFSolver.findVCFSequence(black43Fork, memoBlack)
      VCFSolver.findVCFSequence(whiteTrap, memoWhite)
    }
  }

}
