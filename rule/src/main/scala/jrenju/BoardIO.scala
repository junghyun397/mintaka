package jrenju

import jrenju.notation.{Flag, Opening, Renju}
import utils.lang.{StringArrayTransform, IntTransform}

//noinspection DuplicatedCode
object BoardIO {

  // regex: [0-9][\s(]([^\s][\s()]){15}[0-9]
  def fromBoardText(source: String, latestMove: Int, opening: Option[Opening]): Option[L1Board] = this.fromFieldArray(
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
    opening,
  )

  def fromFieldArray(source: Array[Byte], latestMove: Int, opening: Option[Opening]): Option[L1Board] =
    if (source.length != Renju.BOARD_LENGTH) Option.empty
    else Option(new L1Board(
      boardField = source,
      pointsField = Array.fill(Renju.BOARD_LENGTH)(new PointsPair()),
      moves = source.count {
        case Flag.BLACK => true
        case Flag.WHITE => true
        case _ => false
      },
      latestMove = latestMove,
      opening = opening,
    ))

  implicit class BoardToText(source: Board) {

    private lazy val columnHint: String = f"   ${
      Seq.range(65, 65 + Renju.BOARD_WIDTH)
        .map(idx => f"${idx.toChar} ")
        .mkString
    }  "

    def attributeText[A, B](extract: Board => Array[A])(transform: A => B): String = f"$columnHint\n${
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

    def boardText: String = this.attributeText(_.boardField)(Flag.flagToChar)

    def debugText: String =
      f"${this.boardText}\n" +
        Array(
          f"\nblack-open-3 /\n${this.attributeText(_.pointsField)(_.black.three.dotIfZero)}\n",
          f"\nblack-closed-4 /\n${this.attributeText(_.pointsField)(_.black.closedFour.dotIfZero)}\n",
          f"\nblack-open-4 /\n${this.attributeText(_.pointsField)(_.black.open4.count(_ == true).dotIfZero)}\n",
          f"\nblack-5\n${this.attributeText(_.pointsField)(_.black.fiveInRow.dotIfZero)}\n"
        ).mergeHorizontal +
        Array(
          f"\nwhite-open-3 /\n${this.attributeText(_.pointsField)(_.white.three.dotIfZero)}\n",
          f"\nwhite-closed-4 /\n${this.attributeText(_.pointsField)(_.white.closedFour.dotIfZero)}\n",
          f"\nwhite-open-4 /\n${this.attributeText(_.pointsField)(_.white.open4.count(_ == true).dotIfZero)}\n",
          f"\nwhite-5\n${this.attributeText(_.pointsField)(_.white.fiveInRow.dotIfZero)}\n"
        ).mergeHorizontal

  }

}
