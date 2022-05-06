package utils.lang

import java.util.concurrent.ConcurrentHashMap
import scala.language.implicitConversions

class ConcurrentMapOps[K, V](private val xm: ConcurrentHashMap[K, V]) extends AnyVal {

  def getOrElseUpdate(key: K, f: () => V): V = {
    var value = xm.get(key)
    if (value == null) {
      value = f()
      xm.put(key, value)
    }
    value
  }

}

object ConcurrentMapOps {

  implicit def concurrentMapOps[K, V](map: ConcurrentHashMap[K, V]): ConcurrentMapOps[K, V] = new ConcurrentMapOps[K, V](map)

}
