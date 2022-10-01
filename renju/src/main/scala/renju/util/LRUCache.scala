package renju.util

class LRUCache[K, V](private val maxCapacity: Int)
  extends java.util.LinkedHashMap[K, V](100, 0.75f, true) {

  override def removeEldestEntry(eldest: java.util.Map.Entry[K, V]): Boolean = this.size() > this.maxCapacity

}
