package jrenju

import jrenju.PointOps.pointsOps
import org.scalatest.flatspec._
import org.scalatest.matchers._

import scala.language.implicitConversions

class mm(val i: Int) {

  @inline def dec: Int = i - 1

}

object mm {

  def dec(i: Int): Int = i - 1

  implicit def mm(i: Int): mm = new mm(i)

}

object Playground {

  def main(args: Array[String]): Unit = {
    val startTime = System.currentTimeMillis()

    var i = Int.MaxValue

    while (i > 0) {
//              i = i.dec
      //        i = mm.dec(i)
      i = i - 1
    }

    println(System.currentTimeMillis() - startTime)
  }

}

class Playground extends AnyFlatSpec with should.Matchers {

  "language test while" should "works" in { // 10151 ms
    val startTime = System.currentTimeMillis()

    var i = Int.MaxValue

    var about = 0

    for (_ <- 0 until 1) {
      while (i > 0) {
//        i = i.dec
//        i = mm.dec(i)
        i = i - 1
        about += 1
      }
    }

    println(about)

    println(System.currentTimeMillis() - startTime)
  }

}
