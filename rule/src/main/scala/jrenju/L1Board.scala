package jrenju

import jrenju.notation.Opening

final class L1Board(
  boardField: Array[Byte],
  moves: Int,
  latestMove: Int,
  opening: Option[Opening]
) extends Board(boardField, moves, latestMove, opening) {

  private def composeL2Strips(): Array[L2Strip] = ???

  private def fullComposeL2Strips(): Array[L2Strip] = ???

  private def assembleL2Strips(strips: Array[L2Strip]): (Array[AttackPoints], Boolean) = ???

  private def assembleL2Board(attackField: Array[AttackPoints], isEnd: Boolean): L2Board = new L2Board(
    this.boardField,
    attackField,
    this.moves,
    this.latestMove,
    this.opening,
    isEnd,
  )

  def calculateL2Board(): L2Board = {
    val assembly = assembleL2Strips(composeL2Strips())
    this.assembleL2Board(assembly._1, assembly._2)
  }

  def fullCalculateL2Board(): L2Board = {
    val assembly = assembleL2Strips(fullComposeL2Strips())
    this.assembleL2Board(assembly._1, assembly._2)
  }

}
