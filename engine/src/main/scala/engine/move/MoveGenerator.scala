package engine.move

import renju.Board

trait MoveGenerator {

  def collectValidMoves(board: Board): Array[Int]

}
