//noinspection DuplicatedCode, ScalaUnusedSymbol

package jrenju

import jrenju.Struct.particleOps
import jrenju.notation.{Flag, Pos, Renju}
import utils.lang.Transform.joinHorizontal

import scala.language.implicitConversions

object BoardIO {

  // regex: [a-z][0-9][0-9]?[0-9]?
  def buildPosSequence(source: String): Option[Seq[Pos]] = {
    val seq = "[a-z]\\d\\d?\\d?".r
      .findAllIn(source)
      .map(Pos.fromCartesian)
      .toSeq

    Option.when(!seq.exists(_.isEmpty)) { seq.map(_.get) }
  }

  def fromSequence(source: String): Option[Board] = this.buildPosSequence(source)
    .map(_.map(_.idx))
    .flatMap(this.fromSequence)

  def fromSequence(source: Seq[Int]): Option[Board] = {
    val field = Array.fill(Renju.BOARD_SIZE)(Flag.EMPTY)

    source.zipWithIndex foreach { case (idx, order) =>
      field(idx) =
        if (order % 2 == 0) Flag.BLACK
        else Flag.WHITE
    }

    val board = new ScalaBoard(
      field = field,
      structFieldBlack = Array.fill(Renju.BOARD_SIZE)(0),
      structFieldWhite = Array.fill(Renju.BOARD_SIZE)(0),
      moves = source.length,
      lastMove = source.last,
      winner = Option.empty,
      hashKey = ZobristHash.boardHash(field)
    )

    board.integrateStrips(board.composeGlobalStrips().map(_.calculateL2Strip()))
    board.calculateForbids()

    Some(board)
  }

  def fromBoardText(source: String, latestMove: Int): Option[Board] = {
    val regex = """\d[\s\[](\S[\s\[\]]){15}\d""".r

    this.fromFieldArray(
      regex.findAllIn(source)
        .toArray
        .flatMap(_
          .drop(1)
          .dropRight(1)
          .toUpperCase
          .map(Flag.charToFlag)
          .filter(_.isDefined)
          .map(_.get)
          .reverse
        )
        .reverse,
      latestMove,
    )
  }

  def fromFieldArray(source: Array[Byte], latestMove: Int): Option[Board] =
    if (source.length != Renju.BOARD_SIZE) Option.empty
    else {
      val board = new ScalaBoard(
        field = source,
        structFieldBlack = Array.fill(Renju.BOARD_SIZE)(0),
        structFieldWhite = Array.fill(Renju.BOARD_SIZE)(0),
        moves = source.count {
          case Flag.BLACK | Flag.WHITE => true
          case _ => false
        },
        lastMove = latestMove,
        winner = Option.empty,
        hashKey = ZobristHash.boardHash(source)
      )

      board.integrateStrips(board.composeGlobalStrips().map(_.calculateL2Strip()))
      board.calculateForbids()

      Some(board)
    }

  implicit class BoardToString(b: Board) {

    private lazy val columnHint: String = f"   ${
      Seq.range(65, 65 + Renju.BOARD_WIDTH)
        .map(_.toChar.toString)
        .reduce((acc, col) => f"$acc $col")
        .mkString
    }   "

    def attributeString[T](markLastMove: Boolean)(extract: Board => Array[T])(transform: T => String): String = f"$columnHint\n${
      val result = extract(this.b)
        .grouped(Renju.BOARD_WIDTH)
        .zipWithIndex
        .map { case (col, idx) =>
          f"${idx + 1}%2d ${
            col
              .map(value => transform(value))
              .reduce((acc, elem) => f"$acc $elem")
          } ${idx + 1}%-2d\n"
        }
        .toArray
        .reverse
        .flatten
        .mkString

      if (markLastMove && b.moves != 0) {
        val offset =(Renju.BOARD_WIDTH_MAX_IDX - Pos.idxToRow(b.lastMove)) * (6 + Renju.BOARD_WIDTH * 2) + Pos.idxToCol(b.lastMove) * 2 + 2
        result
          .updated(offset, '[')
          .updated(offset + 2, ']')
      } else result
    }$columnHint"

    def boardString: String = this.boardString(true)

    def boardString(markLatestMove: Boolean): String =
      this.attributeString(markLatestMove)(_.field)(flag => Flag.flagToChar(flag).toString)

    private val pointFieldTextBlack: (Int => String) => String = this.attributeString(markLastMove = false)(_.structFieldBlack)
    private val pointFieldTextWhite: (Int => String) => String = this.attributeString(markLastMove = false)(_.structFieldWhite)

    implicit def dotIfZero(i: Int): String = if (i == 0) "." else i.toString

    def debugString: String =
      f"${this.boardString}\n" +
        joinHorizontal(
          f"\nblack-open-three /\n${this.pointFieldTextBlack(_.threeTotal)}\n",
          f"\nblack-block-three /\n${this.pointFieldTextBlack(_.blockThreeTotal)}\n",
          f"\nblack-closed-four /\n${this.pointFieldTextBlack(_.closedFourTotal)}\n",
          f"\nblack-open-four /\n${this.pointFieldTextBlack(_.openFourTotal)}\n",
          f"\nblack-five\n${this.pointFieldTextBlack(_.fiveTotal)}\n"
        ) +
        joinHorizontal(
          f"\nwhite-open-three /\n${this.pointFieldTextWhite(_.threeTotal)}\n",
          f"\nwhite-block-three /\n${this.pointFieldTextWhite(_.blockThreeTotal)}\n",
          f"\nwhite-closed-four /\n${this.pointFieldTextWhite(_.closedFourTotal)}\n",
          f"\nwhite-open-four /\n${this.pointFieldTextWhite(_.openFourTotal)}\n",
          f"\nwhite-five\n${this.pointFieldTextWhite(_.fiveTotal)}\n"
        )

  }

}
