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

  def buildAttributeText[A, B](source: Array[A])(transform: A => B): String = f"$columnHint\n${
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

  def buildBoardText(boardField: Array[Byte]): String = buildAttributeText(boardField)(Flag.flagToChar)

  def buildDebugText(boardField: Array[Byte], attackField: Array[(AttackPoints, AttackPoints)]): String =
      f"${this.buildBoardText(boardField)}\n" +
      f"\nblack-open-3\n${buildAttributeText(attackField)(_._1.open3.dotIfZero)}\n" +
      f"\nblack-closed-4\n${buildAttributeText(attackField)(_._1.closed4.dotIfZero)}\n" +
      f"\nblack-open-4\n${buildAttributeText(attackField)(_._1.open4.dotIfZero)}\n" +
      f"\nblack-5\n${buildAttributeText(attackField)(_._1.five.dotIfZero)}\n" +
      f"\nwhite-open-3\n${buildAttributeText(attackField)(_._2.open3.dotIfZero)}\n" +
      f"\nwhite-closed-4\n${buildAttributeText(attackField)(_._2.closed4.dotIfZero)}\n" +
      f"\nwhite-open-4\n${buildAttributeText(attackField)(_._2.open4.dotIfZero)}\n" +
      f"\nwhite-5\n${buildAttributeText(attackField)(_._2.five.dotIfZero)}\n"

  def fromBoardText(source: String, latestMove: Int, opening: Option[Opening]): Option[L1Board] = fromBoardFieldArray(
    ("[0-9]\\s([^\\s]\\s){" + Renju.BOARD_WIDTH + "}[0-9]").r.findAllIn(source)
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

  def fromBoardFieldArray(source: Array[Byte], latestMove: Int, opening: Option[Opening]): Option[L1Board] =
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
