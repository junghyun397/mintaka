package jrenju.notation

sealed trait InvalidKind

object InvalidKind {

  case object Exist extends InvalidKind
  case object Forbidden extends InvalidKind

}
