package jrenju

import jrenju.notation.Opening
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

  implicit class T2b(val source: String) {

    def t2b: L1Board = source.t2b(Renju.BOARD_CENTER.idx)

    def t2b(latestMove: Int): L1Board = source.t2b(latestMove, Option.empty)

    def t2b(latestMove: Int, opening: Option[Opening]): L1Board = BoardTransform.fromBoardText(source, latestMove, opening).get

  }

}
