package jrenju.protocol

sealed abstract class Solution(val idx: Int)

final class SolutionNode(idx: Int, val child: Map[Int, Solution]) extends Solution(idx)

final class SolutionLeaf(idx: Int) extends Solution(idx)
