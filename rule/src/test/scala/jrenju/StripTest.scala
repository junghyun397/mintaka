package jrenju

import jrenju.TestHelper.T2
import jrenju.notation.Flag
import org.scalatest.flatspec._
import org.scalatest.matchers._

class StripTest extends AnyFlatSpec with should .Matchers {

  def open3(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open3) "1" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open3) "1" else ".").mkString should be (answer.reverse)
  }

  "open-3 points" should "detect correctly" in {
    // XX

    open3("...XX...", ".11..11.")

    open3("...XX..O...", ".11..1.....")

    open3("...XX.O...", ".11.......")

    open3("...X...XX...", "......1..11.")

    open3("...X...XX.O...", "..............")

    open3("...X...XX..O...", "......1..1.....")

    open3("...O..XX..O...", ".....1..1.....")

    open3("...X...XX...X...", "......1..1......")

    open3("...X...XX..XX..O...", "...................")

    // X.X

    open3("...X.X...", "..1.1.1..")

    open3(".X.X...", "..1.1..")

    open3("...X.X..O...", "..1.1.1.....")

    open3("...X.X.O...", "..1.1......")

    open3("....X..X.X.O...", "...............")

    open3("...X...X.X.O...", "......1.1......")

    open3("...X..X.X..X...", "...............")

    // X..X

    open3("...X..X...", "....11....")

    open3("...X..X.X.O...", "..............")

    open3("...X..X.X..O...", ".......1.1.....")

    open3("...X..X..X...", "....11.11....")

    open3("...X..X.X...", ".......1.1..")

  }

  def closed4(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.closed4 == 0) "." else v.black.closed4.toString).mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.closed4 == 0) "." else v.black.closed4.toString).mkString should be (answer.reverse)
  }

  "closed-4 points" should "detect correctly" in {
    // XXX

    closed4("...XXX...", ".1.....1.")

    closed4("...OXXX...", ".......11.")

    closed4("...O.XXX...", "....1....1.")

    closed4("...O.XXX.O...", "....1...1....")

    closed4("...O.XXX..O...", "....1....1....")

    closed4("...XXX..XXX...", ".1....11....1.")

    closed4("...X.XXX..X...", "..............")

    // XX.X

    closed4("...XX.X...", "..1....1..")

    closed4("...OXX.X...", "......1.1..")

    closed4("...O.XX.X...", "....1....1..")

    closed4("...O.XX.XO...", "....1..1.....")

    closed4("...OXX.X.O...", "......1.1....")

    closed4("...O.XX.XO...", "....1..1.....")

    // X.XX

    closed4("...OX.XX...", ".....1..1..")

    closed4("...O.X.XXO...", "....1.1......")

    closed4("...OX.XX..X...", ".....1...1....")

    closed4("...OXX.X.O...", "......1.1....")

    // XX..X

    closed4("...XX..X...", ".....11....")

    closed4("....XX..XX...", ".............")

    // complex

    closed4("...X.XX.X.X...", "..1.1.........")

    closed4("...OX.XX.X...", "........1.1..")
  }

  def open4(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open4) "1" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.open4) "1" else ".").mkString should be (answer.reverse)
  }

  "open-4 points" should "detect correctly" in {
    // XXX

    open4("...XXX...", "..1...1..")

    open4("...XXX..X...", "..1.........")

    open4("...XXX...XXX...", "..1...1.1...1..")

    open4("...O.XXX...", "........1..")

    open4("...O.XXX.O...", ".............")

    // XX.X

    open4("...XX.X...", ".....1....")

    open4("...OXX.X...", "...........")

    open4("...XX.X.X...", "............")
  }

  def five(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.five) "1" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (v.black.five) "1" else ".").mkString should be (answer.reverse)
  }

  "move-to-win points" should "detect correctly" in {
    five("...XXXX...", "..1....1..")

    five("...OXXXX...", "........1..")

    five("...XX.XX...", ".....1.....")

    five("...XXX.X...", "......1....")

    five("...XXXX.XX...", "..1..........")

    five("...XXX.XX...", "............")

    five("...XXXX..XXXX...", "..1....11....1..")
  }

  def win(problem: String, answer: Byte): Unit = {
    problem.t2s.calculateL2Strip().winner should be (answer)
    problem.reverse.t2s.calculateL2Strip().winner should be (answer)
  }

  "5-in-a-row" should "detect correctly" in {
    win(".XXOX..OXXXX.X", Flag.FREE)

    win("..XOOXO.OOOO.O", Flag.FREE)

    win("XXXXX", Flag.BLACK)

    win("OOOOO", Flag.WHITE)

    win("..OX.XXO.XXXXXO..", Flag.BLACK)

    win("..OOOOX.OXOOOOOX", Flag.WHITE)
  }

  def over6forbid(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_6) "6" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_6) "6" else ".").mkString should be (answer.reverse)
  }

  "over-6 forbidden points" should "detect correctly" in {
    over6forbid("...XXX.XX...", "......6.....")

    over6forbid("...X.XXXX...", "....6.......")

    over6forbid("...X.XXX.XXX...", "........6......")

    over6forbid("...OXXXX.X...", "........6....")
  }

  def double4forbid(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_44) "4" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_44) "4" else ".").mkString should be (answer.reverse)
  }

  "double-4 forbidden points" should "detect correctly" in {
    double4forbid("...X..XX.X...", ".....4.......")

    double4forbid("...XXX...XXX...", ".......4.......")

    double4forbid("...XX..X.XX...", "......4.......")

    double4forbid("...XXX..X.XX...", "...............")

    double4forbid("...X.X.X.X...", "......4......")

    double4forbid("...X.X.X.X.X...", "......4.4......")

    double4forbid("...X.X.X.X.X.X.X.X...", "......4.4.4.4.4......")
  }

}
