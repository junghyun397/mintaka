package jrenju

import jrenju.ComplexCase.T2
import jrenju.notation.Flag
import org.scalatest.flatspec._
import org.scalatest.matchers._

class StripTest extends AnyFlatSpec with should .Matchers {

  private def open3(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open3 == 0) "." else v.black.open3.toString).mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open3 == 0) "." else v.black.open3.toString).mkString should be (answer.reverse)
  }

  "open-3-attack points" should "detect correctly" in {
    open3("...OO...", ".11..11.")

    open3("...O.O...", "..1.1.1..")

    open3("...O..O...", "....11....")

    open3("...O..O..O...", "....11.11....")

    open3("...O..O.O...", ".......1.1..")
  }

  private def closed4(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.closed4 == 0) "." else v.black.closed4.toString).mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.closed4 == 0) "." else v.black.closed4.toString).mkString should be (answer.reverse)
  }

  "closed-4-attack points" should "detect correctly" in {}

  private def open4(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open4 == 0) "." else v.black.open4.toString).mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open4 == 0) "." else v.black.open4.toString).mkString should be (answer.reverse)
  }

  "open-4-attack points" should "detect correctly" in {
    val case1 = "..O"
  }

  private def five(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.five == 0) "." else v.black.five.toString).mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.five == 0) "." else v.black.five.toString).mkString should be (answer.reverse)
  }

  "move-to-win points" should "detect correctly" in {
    val case1 = "..OXOO.OOXO".t2s
    case1.calculateL2Strip().pointsStrip(6).black.five should be (1)

    val case2 = "..OXX.XX.XXOO".t2s
    case2.calculateL2Strip().pointsStrip(8).white.five should be (1)
  }

  "5-in-a-row strip" should "detect correctly" in {
    val case1 = ".OOXO..XOOOO.O".t2s
    case1.calculateL2Strip().winner should be (Flag.FREE)

    val case2 = "..OXXOX.XXXX.X".t2s
    case2.calculateL2Strip().winner should be (Flag.FREE)

    val case3 = "OOOOO".t2s
    case3.calculateL2Strip().winner should be (Flag.BLACK)

    val case4 = "XXXXX".t2s
    case4.calculateL2Strip().winner should be (Flag.WHITE)

    val case5 = "..XO.OOX.OOOOOX..".t2s
    case5.calculateL2Strip().winner should be (Flag.BLACK)

    val case6 = "..XXXXO.XOXXXXXO".t2s
    case6.calculateL2Strip().winner should be (Flag.WHITE)
  }

  private def over6forbid(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_6) "6" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_6) "6" else ".").mkString should be (answer.reverse)
  }

  "over-6 forbidden points" should "detect correctly" in {
    val case1 = "..XOOO.OO.X".t2s
    case1.calculateL2Strip().forbidMask(6) should be (Flag.FORBIDDEN_6)

    val case2 = ".XOOO.OOO..".t2s
    case2.calculateL2Strip().forbidMask(5) should be (Flag.FORBIDDEN_6)

    val case3 = ".XOOOO.OOOO.".t2s
    case3.calculateL2Strip().forbidMask(6) should be (Flag.FORBIDDEN_6)
  }

  private def double4forbid(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_44) "4" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_44) "4" else ".").mkString should be (answer.reverse)
  }

  "double-4 forbidden points" should "detect correctly" in {}

}
