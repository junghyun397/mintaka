package jrenju

import jrenju.notation.{Direction, Flag, Opening, Pos}
import jrenju.rule.Renju

import scala.language.implicitConversions

/*
"   A B C D E F G H I J K L M N O   \n" +
"15 . . . . . . . . . . . . . . X 15\n" +
"14 . . . . . . . . . . . . . . . 14\n" +
"13 . . . . . . . . . . . . . . . 13\n" +
"12 . . . . . . . . . . . . . . . 12\n" +
"11 . . . . . . . . . . . . . . . 11\n" +
"10 . . . . . . . . . . . . . . . 10\n" +
" 9 . . . . . . . . . . . . . . .  9\n" +
" 8 . . . . . . . O . . . . . . .  8\n" +
" 7 . . . . . . . . . . . . . . .  7\n" +
" 6 . . . . . . . . . . . . . . .  6\n" +
" 5 . . . . . . . . . . . . . . .  5\n" +
" 4 . . . . . . . . . . . . . . .  4\n" +
" 3 . . . . . . . . . . . . . . .  3\n" +
" 2 . . . . . . . . . . . . . . .  2\n" +
" 1 . . . . . . . . . . . . . . .  1\n" +
"   A B C D E F G H I J K L M N O   "
 */

object ComplexCase {

  implicit class T2(val source: String) {

    def t2b: L1Board = source.t2b(Renju.BOARD_CENTER.idx)

    def t2b(x: String, y: Int): L1Board = source.t2b(Pos.fromCartesian(x, y).get.idx)

    def t2b(latestMove: Int): L1Board = source.t2b(latestMove, Option.empty)

    def t2b(latestMove: Int, opening: Option[Opening]): L1Board = BoardIO.fromBoardText(source, latestMove, opening).get

    def t2s: L1Strip = new L1Strip(
      Direction.X,
      0,
      source
        .map(Flag.charToFlag)
        .map(_.get)
        .toArray
    )

  }

  val complexForbidden: L1Board = (
    "   A B C D E F G H I J K L M N O   \n" +
    "15 . . . . . . . . . . . . . . X 15\n" +
    "14 . . . . . . . . . . . . . . . 14\n" +
    "13 . . . . . . . . . . . . . . . 13\n" +
    "12 . . . . . X . . . . . . . . . 12\n" +
    "11 . . . . . O X . X . . . . . . 11\n" +
    "10 . . . . . . O . O . X . . . . 10\n" +
    " 9 . . . . X . X O . . . . . . . 9 \n" +
    " 8 . . . . . O X O X . . . . . . 8 \n" +
    " 7 . . . . . O . . O . . . . . . 7 \n" +
    " 6 . . . . . . . . . . . . . . . 6 \n" +
    " 5 . . . . . . . . . . . . . . . 5 \n" +
    " 4 . . . . . . . . . . . . . . . 4 \n" +
    " 3 . . . . . . . . . . . . . . . 3 \n" +
    " 2 . . . . . . . . . . . . . . . 2 \n" +
    " 1 . . . . . . . . . . . . . . . 1 \n" +
    "   A B C D E F G H I J K L M N O   "
    ).t2b("K", 10)

}
