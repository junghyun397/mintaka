package jrenju.solve

import jrenju.Board
import jrenju.protocol.Solution

class VCTSolver {

  implicit class VCTFinder(val board: Board) {

    def findVCTSolution: Option[Solution] = Option.empty

  }

  implicit class VCFFinder(val board: Board) {

    def findVCFSolution: Option[Solution] = Option.empty

  }

}
