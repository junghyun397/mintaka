package jrenju

import jrenju.BoardIO.BoardToText
import jrenju.TestHelper.T2
import jrenju.notation.Pos
import org.scalatest.flatspec._
import org.scalatest.matchers._

class BoardIOTest extends AnyFlatSpec with should.Matchers {

  "from sequence" should "works correctly" in {
    val board = BoardIO.fromSequence("h8,g8,c14,b15,e15,f14,g15,a15,a11,a12,c11,a10,b7,a9,a1,a13,b3,a14,a4,b4,d5,e6,f4,d8,i4,d10,k6,i6,l4,i7,n4,i9,o5,o4,m7,o6,m8,n9,n10,o3,n2,o2,m1,n1,j1,k1,h1,l1,o14,k2,o12,b2,n15,m15,k15,l15,k14,j15,f12,b1,g10,d1,o9,f1")
      .get

    board shouldNot be (Option.empty)
  }

  "from board text" should "injection function" in {
    val board = """
      |
      |""".stripMargin
      .t2b(Pos.fromCartesian("I", 2).get.idx)

    board.boardText should be (BoardIO.fromBoardText(board.boardText, 0).get.boardText)
  }

}
