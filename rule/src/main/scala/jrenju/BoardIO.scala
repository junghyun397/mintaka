package jrenju

import jrenju.notation.{Flag, Pos, Renju}
import jrenju.solve.ZobristHash
import utils.lang.Transform.joinHorizontal

import scala.language.implicitConversions

//noinspection DuplicatedCode
object BoardIO {

  // regex: [a-z][0-9][0-9]?[0-9]?
  def fromPosSequence(source: String): Option[Seq[Pos]] = {
    val seq = "[a-z][0-9][0-9]?[0-9]?".r
      .findAllIn(source)
      .map(Pos.fromCartesian)
      .toSeq

    if (seq.exists(_.isEmpty)) Option.empty
    else Option(seq.map(_.get))
  }

  def fromSequence(source: String): Option[Board] = this.fromPosSequence(source)
    .map(_.map(_.idx))
    .flatMap(this.fromSequence)

  def fromSequence(source: Seq[Int]): Option[Board] = {
    val field = Array.fill(Renju.BOARD_LENGTH)(Flag.FREE)

    source.zipWithIndex foreach { idxOrder =>
      field(idxOrder._1) =
        if (idxOrder._2 % 2 == 0) Flag.BLACK
        else Flag.WHITE
    }

    val board = new Board(
      boardField = field,
      pointsField = Array.fill(Renju.BOARD_LENGTH)(new PointsPair()),
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
    if (source.length != Renju.BOARD_LENGTH) Option.empty
    else {
      val board = new Board(
        boardField = source,
        pointsField = Array.fill(Renju.BOARD_LENGTH)(new PointsPair()),
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

    private val structText: (PointsPair => String) => String = this.attributeText(_.pointsField)

    implicit def dotIfZero(i: Int): String = if (i == 0) "." else i.toString

    def debugText: String =
      f"${this.boardText}\n" +
        joinHorizontal(
          f"\nblack-open-3 /\n${this.structText(_.black.three)}\n",
          f"\nblack-block-3 /\n${this.structText(_.black.block3.count(_ == true))}\n",
          f"\nblack-closed-4 /\n${this.structText(_.black.closedFour)}\n",
          f"\nblack-open-4 /\n${this.structText(_.black.open4.count(_ == true))}\n",
          f"\nblack-5\n${this.structText(_.black.fiveInRow)}\n"
        ) +
        joinHorizontal(
          f"\nwhite-open-3 /\n${this.structText(_.white.three)}\n",
          f"\nwhite-block-3 /\n${this.structText(_.white.block3.count(_ == true))}\n",
          f"\nwhite-closed-4 /\n${this.structText(_.white.closedFour)}\n",
          f"\nwhite-open-4 /\n${this.structText(_.white.open4.count(_ == true))}\n",
          f"\nwhite-5\n${this.structText(_.white.fiveInRow)}\n"
        )

  }

}
