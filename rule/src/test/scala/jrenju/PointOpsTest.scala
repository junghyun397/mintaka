package jrenju

import jrenju.BoardIO.BoardToText
import jrenju.PointOps.pointsOps
import jrenju.TestHelper.T2
import jrenju.notation.Direction
import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should
import utils.lang.Transform.IntTransform

class PointOpsTest extends AnyFlatSpec with should.Matchers {

  def test(problem: String): Unit = {
    println(problem.t2b.debugText)
  }

  "test" should "test" in {
    test(
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
        | 6 . . . . . X X . . . . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
    )
  }

  "merge that" should "works" in {
    val original = 0x40000000
    val that = 0x80000000
    val rs = original.merged(Direction.X, that)
    println(original.toSeparatedBinaryString)
    println(that.toSeparatedBinaryString)
    println(rs.toSeparatedBinaryString)
    println(rs.threeTotal)
  }

}
