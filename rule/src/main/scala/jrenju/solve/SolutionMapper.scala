package jrenju.solve

import jrenju.protocol.{Solution, SolutionLeaf, SolutionNode}

object SolutionMapper {

  implicit class SequenceToNode(sequence: Seq[Int]) {

    def toSolution: Solution = {
      val leaf = new SolutionLeaf(this.sequence.last)

      this.sequence
        .dropRight(1)
        .grouped(2)
        .foldRight[Solution](leaf) { (movePair, child) =>
          new SolutionNode(movePair.head, Map(movePair.last -> child))
        }
    }

  }

}
