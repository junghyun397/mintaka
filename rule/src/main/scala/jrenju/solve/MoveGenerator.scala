package jrenju.solve

import jrenju.Board

trait MoveGenerator {

  def collectValidMoves(board: Board): Array[Int]

}
