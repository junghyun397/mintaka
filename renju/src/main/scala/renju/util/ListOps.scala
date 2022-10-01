package renju.util

import java.util
import scala.language.implicitConversions

class ListOps[T](private val xs: List[T]) extends AnyVal {

  def asJavaList: java.util.List[T] = {
    val arrayList = new util.ArrayList[T]()

    xs.foreach(arrayList.add)

    arrayList
  }

}

object ListOps {

  implicit def listOps[T](list: List[T]): ListOps[T] = new ListOps[T](list)

}
