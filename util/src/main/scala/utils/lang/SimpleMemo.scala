package utils.lang

import scala.collection.mutable

object SimpleMemo {

  def memoize[I, O](f: I => O): I => O =
    new mutable.HashMap[I, O]() { self =>
      override def apply(key: I): O = self.getOrElseUpdate(key, f(key))
    }

  def memoizeSync[I, O](f: I => O): I => O =
    new mutable.HashMap[I, O]() { self =>
      override def apply(key: I): O = self.synchronized(getOrElseUpdate(key, f(key)))
    }

}
