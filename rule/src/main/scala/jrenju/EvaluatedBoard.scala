package jrenju

import jrenju.notation.{Color, Flag}

trait EvaluatedBoard {

  val winner: Byte

  def isEnd: Boolean = winner != Flag.FREE

  def winnerColor: Option[Color.Value] = if (this.isEnd) Option(Color(winner)) else Option.empty

}
