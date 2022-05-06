package utils.lang

import java.util.Collections.synchronizedMap

class LRUCache[K, V](private val maxCapacity: Int)
  extends java.util.LinkedHashMap[K, V](100, 0.75f, true) {

  override def removeEldestEntry(eldest: java.util.Map.Entry[K, V]): Boolean = this.size() > this.maxCapacity

}

object LRUCache {

  def apply[K, V](maxCapacity: Int): java.util.Map[K, V] = synchronizedMap(new LRUCache[K, V](maxCapacity))

}
