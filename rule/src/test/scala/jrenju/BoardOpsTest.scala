package jrenju

import jrenju.TestHelper.T2
import jrenju.notation.Flag
import org.scalatest.flatspec._
import org.scalatest.matchers._

class BoardOpsTest extends AnyFlatSpec with should.Matchers {

  def win(problem: String, answer: Option[Byte]): Unit = {
    problem.t2b.winner should be (answer)
  }

  "five-in-a-row board" should "has winner" in {
    win(
      """
        |   A B C D E F G H I J K L M N O
        |15 . . . . . . . . . . . . . . . 15
        |14 . . . . . . . . . . . . . . . 14
        |13 . . . . . . . . . . . . . . . 13
        |12 . . . . . . . . . . . . . . . 12
        |11 . . . . . . . . . . . . . . . 11
        |10 . . . . . . . . . . . . . . . 10
        | 9 . . . . . . . . . . . . . . . 9
        | 8 . . . . . X X X X X . . . . . 8
        | 7 . . . . . . . . . . . . . . . 7
        | 6 . . . . . . . . . . . . . . . 6
        | 5 . . . . . . . . . . . . . . . 5
        | 4 . . . . . . . . . . . . . . . 4
        | 3 . . . . . . . . . . . . . . . 3
        | 2 . . . . . . . . . . . . . . . 2
        | 1 . . . . . . . . . . . . . . . 1
        |   A B C D E F G H I J K L M N O
      """.stripMargin,
      Some(Flag.BLACK)
    )
  }

}
