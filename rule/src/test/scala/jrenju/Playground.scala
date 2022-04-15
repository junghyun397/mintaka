package jrenju

import org.scalatest._
import org.scalatest.flatspec._
import org.scalatest.matchers._

class Playground extends AnyFlatSpec with should.Matchers {

  "language test div" should "works" in { // 20662 ms
    println((-1 + 1) % 2)
  }

  "language test foreach" should "works" in { // 20662 ms
    var count = 0

    while (count < 1000000) {
      var reg = (0, 5)
      Array.fill(225)(new PointsPair()).zipWithIndex foreach { zip =>
        reg = (zip._2, zip._1.black.closed4.sum)
      }
      count += 1
    }
  }

  "language test while" should "works" in { // 10151 ms
    var count = 0

    while (count < 1000000) {
      var reg = (0, 5)
      val v = Array.fill(225)(new PointsPair())

      var icount = 0
      while (icount < 225) {
        reg = (icount, v(icount).black.closed4.sum)
        icount += 1
      }
      count += 1
    }
  }

}
