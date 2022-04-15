package jrenju.notation

import org.scalatest._
import flatspec._
import matchers._

class PosTest extends AnyFlatSpec with should.Matchers {

  "Invalid cartesian position" should "return empty" in {
    Pos.fromCartesian(1, "뷁".charAt(0)) should be (Option.empty)
    Pos.fromCartesian(0, "a".charAt(0)) should be (Option.empty)
    Pos.fromCartesian(1, "p".charAt(0)) should be (Option.empty)
  }

  "Valid cartesian position" should "return some pos" in {
    Pos.fromCartesian(1, "a".charAt(0)) should be (Option(Pos(0, 0)))
    Pos.fromCartesian(1, "A".charAt(0)) should be (Option(Pos(0, 0)))
    Pos.fromCartesian(15, "o".charAt(0)) should be (Option(Pos(14, 14)))
  }

}
