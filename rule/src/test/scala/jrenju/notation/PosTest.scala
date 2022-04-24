package jrenju.notation

import org.scalatest._
import flatspec._
import matchers._

class PosTest extends AnyFlatSpec with should.Matchers {

  "Invalid cartesian position" should "return empty" in {
    Pos.fromCartesian("뷁".charAt(0), 1) should be (Option.empty)
    Pos.fromCartesian("a".charAt(0), 0) should be (Option.empty)
    Pos.fromCartesian("p".charAt(0), 1) should be (Option.empty)
  }

  "Valid cartesian position" should "return some pos" in {
    Pos.fromCartesian("a".charAt(0), 1) should be (Option(Pos(0, 0)))
    Pos.fromCartesian("A".charAt(0), 1) should be (Option(Pos(0, 0)))
    Pos.fromCartesian("o".charAt(0), 15) should be (Option(Pos(14, 14)))
  }

}
