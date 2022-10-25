package renju

import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should
import renju.notation.{Direction, Flag, Pos, Renju}

import scala.io.StdIn.readLine
import scala.language.implicitConversions

/*
   A B C D E F G H I J K L M N O
15 . . . . . . . . . . . . . . . 15
14 . . . . . . . . . . . . . . . 14
13 . . . . . . . . . . . . . . . 13
12 . . . . . . . . . . . . . . . 12
11 . . . . . . . . . . . . . . . 11
10 . . . . . . . . . . . . . . . 10
 9 . . . . . . . . . . . . . . . 9
 8 . . . . . . . X . . . . . . . 8
 7 . . . . . . . . . . . . . . . 7
 6 . . . . . . . . . . . . . . . 6
 5 . . . . . . . . . . . . . . . 5
 4 . . . . . . . . . . . . . . . 4
 3 . . . . . . . . . . . . . . . 3
 2 . . . . . . . . . . . . . . . 2
 1 . . . . . . . . . . . . . . . 1
   A B C D E F G H I J K L M N O
 */

object TestHelper {

  implicit class S2(val source: String) {

    def s2p: Pos = Pos.fromCartesian(source).get

    def s2pi: Int = source.s2p.idx

    def s2b: Board = source.s2b(Renju.BOARD_CENTER_POS.idx)

    def s2b(latestMove: Int): Board = BoardIO.fromBoardText(source, latestMove).get

    def s2s: L1Strip = new L1Strip(
      Direction.X,
      0,
      source.length,
      source
        .map { char => Flag.charToFlag(char).get }
        .toArray
    )

  }

}

class TestHelper extends AnyFlatSpec with should.Matchers {

  "board from history text" should "run" in {
    val source = readLine()
    
    val history = """\d{1,3}""".r.findAllIn(source)
      .map(_.toInt)
      .toSeq

    val board = BoardIO.fromIdxSequence(history)

    println(board)
  }

}
