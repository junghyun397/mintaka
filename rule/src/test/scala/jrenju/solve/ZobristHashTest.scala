package jrenju.solve

import jrenju.TestHelper.T2
import jrenju.ZobristHash
import jrenju.notation.Flag
import jrenju.ZobristHash.IncrementHash
import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should

class ZobristHashTest extends AnyFlatSpec with should.Matchers {

  "increment hash" should "same" in {
    println(ZobristHash.empty.incrementHash("h8".t2i, Flag.BLACK))
  }

}
