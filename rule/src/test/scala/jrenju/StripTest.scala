package jrenju

import jrenju.ComplexCase.T2
import jrenju.notation.Flag
import org.scalatest.flatspec._
import org.scalatest.matchers._

class StripTest extends AnyFlatSpec with should .Matchers {

  private def open3(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open3) "1" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open3) "1" else ".").mkString should be (answer.reverse)
  }

  "open-3 points" should "detect correctly" in {
    // OO

    open3("...OO...", ".11..11.")

    open3("...OO..X...", ".11..1.....")

    open3("...OO.X...", ".11.......")

    open3("...O...OO...", "......1..11.")

    open3("...O...OO.X...", "..............")

    open3("...O...OO..X...", "......1..1.....")

    open3("...X..OO..X...", ".....1..1.....")

    open3("...O...OO...O...", "......1..1......")

    open3("...O...OO..OO..X...", "...................")

    // O.O

    open3("...O.O...", "..1.1.1..")

    open3(".O.O...", "..1.1..")

    open3("...O.O..X...", "..1.1.1.....")

    open3("...O.O.X...", "..1.1......")

    open3("....O..O.O.X...", "...............")

    open3("...O...O.O.X...", "......1.1......")

    open3("...O..O.O..O...", "...............")

    // O..O

    open3("...O..O...", "....11....")

    open3("...O..O.O.X...", "..............")

    open3("...O..O.O..X...", ".......1.1.....")

    open3("...O..O..O...", "....11.11....")

    open3("...O..O.O...", ".......1.1..")

  }

  private def closed4(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.closed4 == 0) "." else v.black.closed4.toString).mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.closed4 == 0) "." else v.black.closed4.toString).mkString should be (answer.reverse)
  }

  "closed-4 points" should "detect correctly" in {
    // OOO

    closed4("...OOO...", ".1.....1.")

    closed4("...XOOO...", ".......11.")

    closed4("...X.OOO...", "....1....1.")

    closed4("...X.OOO..X...", "....1....1....")

    closed4("...OOO..OOO...", ".1....11....1.")

    closed4("...O.OOO..O...", "..............")

    // OO.O

    closed4("...OO.O...", "..1....1..")

    closed4("...XOO.O...", "......1.1..")

    closed4("...X.OO.O...", "....1....1..")

    closed4("...X.OO.OX...", "....1..1.....")

    closed4("...XOO.O.X...", "......1.1....")

    // O.OO

    closed4("...XO.OO...", ".....1..1..")

    closed4("...X.O.OOX...", "....1.1......")

    closed4("...XO.OO..O...", ".....1...1....")

    // OO..O

    closed4("...OO..O...", ".....11....")

    closed4("....OO..OO...", ".............")

  }

  private def open4(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open4) "1" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open4) "1" else ".").mkString should be (answer.reverse)
  }

  "open-4 points" should "detect correctly" in {
    // OOO

    open4("...OOO...", "..1...1..")

    open4("...OOO..O...", "..1.........")

    open4("...OOO...OOO...", "..1...1.1...1..")

    open4("...X.OOO...", "........1..")

    open4("...X.OOO.X...", ".............")

    // OO.O

    open4("...OO.O...", ".....1....")

    open4("...XOO.O...", "...........")

    open4("...OO.O.O...", "............")
  }

  private def five(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.five) "1" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.five) "1" else ".").mkString should be (answer.reverse)
  }

  "move-to-win points" should "detect correctly" in {
    five("...OOOO...", "..1....1..")

    five("...XOOOO...", "........1..")

    five("...OO.OO...", ".....1.....")

    five("...OOO.O...", "......1....")

    five("...OOOO.OO...", "..1..........")

    five("...OOO.OO...", "............")

    five("...OOOO..OOOO...", "..1....11....1..")
  }

  "5-in-a-row" should "detect correctly" in {
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
    over6forbid("...OOO.OO...", "......6.....")

    over6forbid("...O.OOOO...", "....6.......")

    over6forbid("...O.OOO.OOO...", "........6......")
  }

  private def double4forbid(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_44) "4" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_44) "4" else ".").mkString should be (answer.reverse)
  }

  "double-4 forbidden points" should "detect correctly" in {
    double4forbid("...O..OO.O...", ".....4.......")

    double4forbid("...OOO...OOO...", ".......4.......")

    double4forbid("...OO..O.OO...", "......4.......")

    double4forbid("...OOO..O.OO...", "...............")

    double4forbid("...O.O.O.O...", "......4......")

    double4forbid("...O.O.O.O.O...", "......4.4......")

    double4forbid("...O.O.O.O.O.O.O.O...", "......4.4.4.4.4......")
  }

}
