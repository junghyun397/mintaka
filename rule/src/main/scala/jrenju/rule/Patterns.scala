package jrenju.rule

object Patterns {

  @inline private val O: Boolean = true
  @inline private val X: Boolean = false

  @inline val THREE_CASE_1: Array[Boolean] = Array(O, O, X, X)
  @inline val THREE_CASE_2: Array[Boolean] = Array(O, X, O, X)

  @inline val THREE_CASE_M: Array[Boolean] = Array(X, O, O, X)

  @inline val FOUR_CASE_1: Array[Boolean] = Array(O, O, X, X, O)
  @inline val FOUR_CASE_2: Array[Boolean] = Array(O, O, X, O, X)
  @inline val FOUR_CASE_3: Array[Boolean] = Array(O, O, O, X, X)

  @inline val FOUR_CASE_M: Array[Boolean] = Array(O, X, O, X, O)

}
