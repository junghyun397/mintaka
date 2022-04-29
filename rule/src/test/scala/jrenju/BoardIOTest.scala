package jrenju

import jrenju.BoardIO.BoardToText
import jrenju.TestHelper.T2
import jrenju.notation.Pos
import org.scalatest.flatspec._
import org.scalatest.matchers._

class BoardIOTest extends AnyFlatSpec with should.Matchers {

  "Board IO" should "injection function" in {
    val board = """
      |
      |""".stripMargin
      .t2b(Pos.fromCartesian("I", 2).get.idx)

    board.boardText should be (BoardIO.fromBoardText(board.boardText, 0).get.boardText)
  }

}
