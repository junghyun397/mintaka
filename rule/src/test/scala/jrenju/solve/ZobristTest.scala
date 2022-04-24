package jrenju.solve

import jrenju.TestHelper.T2
import jrenju.solve.Zobrist.IncrementHash
import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should

class ZobristTest extends AnyFlatSpec with should.Matchers {

  "increment hash" should "same" in {
    println(Zobrist.empty.incrementHash("h8".t2i, isBlack = true))
  }

}
