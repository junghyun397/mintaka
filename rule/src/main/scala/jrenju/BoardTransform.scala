package jrenju

import jrenju.notation.{Flag, Opening}
import jrenju.rule.Renju
import utils.lang.IntTransform

object BoardTransform {

  private lazy val columnHint: String = f"   ${
    Seq.range(65, 65 + Renju.BOARD_WIDTH)
      .map(idx => f"${idx.toChar} ")
      .mkString
  }  "

  def buildTraitText[A, B](source: Array[A])(transform: A => B): String = f"$columnHint\n${
    source
      .grouped(Renju.BOARD_WIDTH)
      .zipWithIndex
      .map(rowIdx => f"${rowIdx._2 + 1}%2d ${
        rowIdx._1
          .map(value => f"${transform(value)} ")
          .mkString
      }${rowIdx._2 + 1}%-2d\n")
      .toArray
      .reverse
      .flatten
      .mkString
  }$columnHint"

  def buildBoardText(boardField: Array[Byte]): String = buildTraitText(boardField)(Flag.flagToChar)

  def buildDebugText(boardField: Array[Byte], attackField: Array[AttackPoints]): String =
      f"${this.buildBoardText(boardField)}\n" +
      f"\nblack-open-3\n${buildTraitText(attackField)(_.black3.dotIfZero)}\n" +
      f"\nblack-closed-4\n${buildTraitText(attackField)(_.blackC4.dotIfZero)}\n" +
      f"\nblack-open-4\n${buildTraitText(attackField)(_.blackO4.dotIfZero)}\n" +
      f"\nblack-5\n${buildTraitText(attackField)(_.black5.dotIfZero)}\n" +
      f"\nwhite-open-3\n${buildTraitText(attackField)(_.white3.dotIfZero)}\n" +
      f"\nwhite-closed-4\n${buildTraitText(attackField)(_.whiteC4.dotIfZero)}\n" +
      f"\nwhite-open-4\n${buildTraitText(attackField)(_.whiteO4.dotIfZero)}\n" +
      f"\nwhite-5\n${buildTraitText(attackField)(_.white5.dotIfZero)}\n"

  def fromBoardText(source: String, latestMove: Int, opening: Option[Opening]): Option[L1Board] = fromBoardByteArray(
    source
      .drop(columnHint.length + 1)
      .dropRight(columnHint.length)
      .split("\n")
      .flatMap(_
        .drop(2)
        .dropRight(2)
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

  def fromBoardByteArray(source: Array[Byte], latestMove: Int, opening: Option[Opening]): Option[L1Board] =
    if (source.length != Renju.BOARD_LENGTH) Option.empty
    else Option.apply(new L1Board(
      boardField = source,
      moves = source.count {
        case Flag.BLACK => true
        case Flag.WHITE => true
        case _ => false
      },
      latestMove = latestMove,
      opening = opening
    ))

}
