package jrenju

import jrenju.ParticleOps.particleOps
import jrenju.notation.{Flag, Pos, Renju}
import utils.lang.Transform.joinHorizontal

import scala.language.implicitConversions

//noinspection DuplicatedCode
object BoardIO {

  // regex: [a-z][0-9][0-9]?[0-9]?
  def buildPosSequence(source: String): Option[Seq[Pos]] = {
    val seq = "[a-z][0-9][0-9]?[0-9]?".r
      .findAllIn(source)
      .map(Pos.fromCartesian)
      .toSeq

    if (seq.exists(_.isEmpty)) Option.empty
    else Option(seq.map(_.get))
  }

  def fromSequence(source: String): Option[Board] = this.buildPosSequence(source)
    .map(_.map(_.idx))
    .flatMap(this.fromSequence)

  def fromSequence(source: Seq[Int]): Option[Board] = {
    val field = Array.fill(Renju.BOARD_SIZE)(Flag.FREE)

    source.zipWithIndex foreach { idxOrder =>
      field(idxOrder._1) =
        if (idxOrder._2 % 2 == 0) Flag.BLACK
        else Flag.WHITE
    }

    val board = new Board(
      boardField = field,
      structFieldBlack = Array.fill(Renju.BOARD_SIZE)(0),
      structFieldWhite = Array.fill(Renju.BOARD_SIZE)(0),
      moves = source.length,
      latestMove = source.last,
      winner = Option.empty,
      zobristKey = ZobristHash.boardHash(field)
    )

    board.integrateStrips(board.composeGlobalStrips())
    board.calculateForbids()

    Option(board)
  }

  // regex: [0-9][\s(]([^\s][\s()]){15}[0-9]
  def fromBoardText(source: String, latestMove: Int): Option[Board] = this.fromFieldArray(
    ("[0-9][\\s(]([^\\s()]\\s){" + Renju.BOARD_WIDTH + "}[0-9]").r.findAllIn(source)
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

  def fromFieldArray(source: Array[Byte], latestMove: Int): Option[Board] =
    if (source.length != Renju.BOARD_SIZE) Option.empty
    else {
      val board = new Board(
        boardField = source,
        structFieldBlack = Array.fill(Renju.BOARD_SIZE)(0),
        structFieldWhite = Array.fill(Renju.BOARD_SIZE)(0),
        moves = source.count {
          case Flag.BLACK | Flag.WHITE => true
          case _ => false
        },
        latestMove = latestMove,
        winner = Option.empty,
        zobristKey = ZobristHash.boardHash(source)
      )

      board.integrateStrips(board.composeGlobalStrips())
      board.calculateForbids()

      Option(board)
    }

  implicit class BoardToText(source: Board) {

    private lazy val columnHint: String = f"   ${
      Seq.range(65, 65 + Renju.BOARD_WIDTH)
        .map(idx => f"${idx.toChar} ")
        .mkString
    }  "

    def attributeText[T](extract: Board => Array[T])(transform: T => String): String = f"$columnHint\n${
      extract(this.source)
        .grouped(Renju.BOARD_WIDTH)
        .zipWithIndex
        .map(colIdx => f"${colIdx._2 + 1}%2d ${
          colIdx._1
            .map(value => f"${transform(value)} ")
            .mkString
        }${colIdx._2 + 1}%-2d\n")
        .toArray
        .reverse
        .flatten
        .mkString
    }$columnHint"

    def boardText: String = this.attributeText(_.boardField)(flag => Flag.flagToChar(flag).toString)

    private val pointFieldTextBlack: (Int => String) => String = this.attributeText(_.structFieldBlack)
    private val pointFieldTextWhite: (Int => String) => String = this.attributeText(_.structFieldWhite)

    implicit def dotIfZero(i: Int): String = if (i == 0) "." else i.toString

    def debugText: String =
      f"${this.boardText}\n" +
        joinHorizontal(
          f"\nblack-three /\n${this.pointFieldTextBlack(_.threeTotal)}\n",
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
