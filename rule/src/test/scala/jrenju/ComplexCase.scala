package jrenju

import jrenju.notation.Pos

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

final case class ComplexCase(board: Board, solution: DeepL3Board)

object ComplexCase {

  implicit class T2b(val s: String) {

    def t2b: L1Board = BoardTransform.fromBoardText(
      source = "",
      latestMove = Pos(7, 7).idx,
      opening = Option.empty
    ).get

  }

  val OPEN_THREE: ComplexCase = ???
  val CLOSED_THREE: ComplexCase = ???

  val OPEN_FOUR: ComplexCase = ???
  val CLOSED_FOUR: ComplexCase = ???

  val DOUBLE_3_FORBIDDEN: ComplexCase = ???
  val DOUBLE_4_FORBIDDEN: ComplexCase = ???
  val OVER_6_FORBIDDEN: ComplexCase = ???

}
