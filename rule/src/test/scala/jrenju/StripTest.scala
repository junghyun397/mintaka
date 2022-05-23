package jrenju

import jrenju.TestHelper.T2
import jrenju.notation.Flag
import org.scalatest.flatspec._
import org.scalatest.matchers._

class StripTest extends AnyFlatSpec with should .Matchers {

  def open3(problem: String, answer: String, op: PointsProvidePair => PointsProvider): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (op(v).open3) "1" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (op(v).open3) "1" else ".").mkString should be (answer.reverse)
  }

  "open-3 points" should "detect correctly" in {
    // XX

    open3("...XX...", ".11..11.", _.black)

    open3("...XX..X...", ".11........", _.black)

    open3("...O..XX..X...", ".....1........", _.black)

    open3("...XX..O...", ".11..1.....", _.black)

    open3("...XX.O...", ".11.......", _.black)

    open3("...X...XX...", "......1..11.", _.black)

    open3("...X...XX.O...", "..............", _.black)

    open3("...X...XX..O...", "......1..1.....", _.black)

    open3("...O..XX..O...", ".....1..1.....", _.black)

    open3("...X...XX...X...", "......1..1......", _.black)

    open3("...X...XX..XX..O...", "...................", _.black)

    // X.X

    open3("...X.X...", "..1.1.1..", _.black)

    open3("...O.X.X..O...", "......1.1.....", _.black)

    open3(".X.X...", "..1.1..", _.black)

    open3("...X.X..O...", "..1.1.1.....", _.black)

    open3("...X.X.O...", "..1.1......", _.black)

    open3("....X..X.X.O...", "...............", _.black)

    open3("...X...X.X.O...", "......1.1......", _.black)

    open3("...X..X.X..X...", "...............", _.black)

    // X..X

    open3("...X..X...", "....11....", _.black)

    open3("...X..X.X.O...", "..............", _.black)

    open3("...X..X.X..O...", ".......1.1.....", _.black)

    open3("...X..X..X...", "....11.11....", _.black)

    open3("...X..X.X...", ".......1.1..", _.black)

  }

  def block3(problem: String, answer: String, op: PointsProvidePair => PointsProvider): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (op(v).block3) "1" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (op(v).block3) "1" else ".").mkString should be (answer.reverse)
  }

//  "block-3 points" should "detect correctly" in {
//    block3("...XXX...", "..1...1..", _.black)
//
//    block3("...O.XXX...", "....1...11.", _.black)
//
//    block3("...O.XXX..O...", "....1...11....", _.black)
//
//    block3("...XXX..X...", ".11...1.....", _.black)
//
//    block3("...XX.X...", "..1..1.1..", _.black)
//  }

  def closed4(problem: String, answer: String, op: PointsProvidePair => PointsProvider): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (op(v).closed4 == 0) "." else op(v).closed4.toString).mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (op(v).closed4 == 0) "." else op(v).closed4.toString).mkString should be (answer.reverse)
  }

  "closed-4 points" should "detect correctly" in {
    // XXX

    closed4("...XXX...", ".1.....1.", _.black)

    closed4("...OXXX...", ".......11.", _.black)

    closed4("...O.XXX...", "....1....1.", _.black)

    closed4("...O.XXX.O...", "....1...1....", _.black)

    closed4("...O.XXX..O...", "....1....1....", _.black)

    closed4("...XXX..XXX...", ".1....11....1.", _.black)

    closed4("...X.XXX..X...", "..............", _.black)

    // XX.X

    closed4("...XX.X...", "..1....1..", _.black)

    closed4("...OXX.X...", "......1.1..", _.black)

    closed4("...O.XX.X...", "....1....1..", _.black)

    closed4("...O.XX.XO...", "....1..1.....", _.black)

    closed4("...OXX.X.O...", "......1.1....", _.black)

    closed4("...O.XX.XO...", "....1..1.....", _.black)

    // X.XX

    closed4("...OX.XX...", ".....1..1..", _.black)

    closed4("...O.X.XXO...", "....1.1......", _.black)

    closed4("...OX.XX..X...", ".....1..21....", _.black)

    closed4("...OXX.X.O...", "......1.1....", _.black)

    // XX..X

    closed4("...XX..X...", ".....11....", _.black)

    closed4("....XX..XX...", ".............", _.black)

    closed4("...XX..X..X", ".....11....", _.black)

    closed4("...XX..X", ".....11.", _.black)

    // complex

    closed4("...X.XX.X.X...", "..1.1.........", _.black)

    closed4("...OX.XX.X...", "........1.1..", _.black)

    closed4("...O.XXX..O...", "....1....1....", _.black)

    closed4("...XX.X.X...", "..1..1......", _.black)
  }

  "white closed-4 points" should "detect correctly" in {
    // WHITE DOUBLE 4 FORK

    closed4("...O.OO..OO.O...", "..1....22....1..", _.white)

    closed4("...OOO...OOO...", ".1.....2.....1.", _.white)

    closed4("...OO..O.OO...", ".....12....1..", _.white)

    closed4("...O.O.O.O...", "....1.2.1....", _.white)

    closed4("...O.O.O.O.O...", "....1.2.2.1....", _.white)

    closed4("...OOO..OO.OX...", ".1....12..1.....", _.white)

    // CLOSED 4

    closed4("OO.OO.", ".....1", _.white)

    closed4("XO.OO.O..", "..1..1.1.", _.white)

    closed4("...XOOO..OOOX...", ".......11.......", _.white)

    closed4("...X.OOO..OOOX...", "....1...11.......", _.white)

    closed4("...XO.O.OOX...", ".....1.1......", _.white)

    closed4("...XOOO..OOO.X...", ".......11...1....", _.white)

    closed4("..XOOO..OX...", "......11.....", _.white)

    closed4("...OOO...", ".1.....1.", _.white)

    closed4("...O.OOO..O...", "..1.....11....", _.white)

    closed4("...XOOO...", ".......11.", _.white)
  }

  def open4(problem: String, answer: String, op: PointsProvidePair => PointsProvider): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (op(v).open4) "1" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (op(v).open4) "1" else ".").mkString should be (answer.reverse)
  }

  "open-4 points" should "detect correctly" in {
    // XXX

    open4("...XXX...", "..1...1..", _.black)

    open4("...XXX..X...", "..1.........", _.black)

    open4("...OOO..O...", "..1...1.....", _.white)

    open4("...XXX...XXX...", "..1...1.1...1..", _.black)

    open4("...O.XXX...", "........1..", _.black)

    open4("...O.XXX.O...", ".............", _.black)

    open4("...O.XXX..O...", "........1.....", _.black)

    // XX.X

    open4("...XX.X...", ".....1....", _.black)

    open4("...OXX.X...", "...........", _.black)

    open4("...XX.X.X...", "............", _.black)

    open4("...OO.O.O...", ".....1......", _.white)
  }

  def five(problem: String, answer: String, op: PointsProvidePair => PointsProvider): Unit = {
    problem.t2s.calculateL2Strip().pointsStrip.map(v => if (op(v).five) "1" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().pointsStrip.map(v => if (op(v).five) "1" else ".").mkString should be (answer.reverse)
  }

  "move-to-win points" should "detect correctly" in {
    five("...XXXX...", "..1....1..", _.black)

    five("...OXXXX...", "........1..", _.black)

    five("...XX.XX...", ".....1.....", _.black)

    five("...XXX.X...", "......1....", _.black)

    five("...XXXX.XX...", "..1..........", _.black)

    five("...XXX.XX...", "............", _.black)

    five("...OOO.OO...", "......1.....", _.white)

    five("...XXXX..XXXX...", "..1....11....1..", _.black)
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

  def double4forbid(problem: String, answer: String): Unit = {
    problem.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_44) "4" else ".").mkString should be (answer)
    problem.reverse.t2s.calculateL2Strip().forbidMask.map(v => if (v == Flag.FORBIDDEN_44) "4" else ".").mkString should be (answer.reverse)
  }

  "double-4 forbidden points" should "detect correctly" in {
    double4forbid("...X.XX..X...", ".......4.....")

    double4forbid("...X..XX.X...", ".....4.......")

    double4forbid("...XXX...XXX...", ".......4.......")

    double4forbid("...XX..X.XX...", "......4.......")

    double4forbid("...XXX..X.XX...", "...............")

    double4forbid("...X.X.X.XX...", "..............")

    double4forbid("...X.X.X.X...", "......4......")

    double4forbid("...X.X.X.X.X...", "......4.4......")

    double4forbid("...X.X.X.X.X.X.X.X...", "......4.4.4.4.4......")
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

}
