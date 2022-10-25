package engine.cache

import engine.cache.TranspositionTable.BUCKET_SIZE
import renju.hash.HashKey

// Transposition Table with Zobrist Hash
// total size = tableSize * BUCKET_SIZE * 16bytes, 1KB = 64pairs
class TranspositionTable(
  val tableSize: Int
) {

  private val totalSize: Int = this.tableSize * BUCKET_SIZE

  private val keys: Array[Long] = new Array[Long](this.totalSize)
  private val entries: Array[Long] = new Array[Long](this.totalSize)

  def write(hashKey: HashKey, eval: Int = 0, score: Int = 0, bestMove: Int = 0, depth: Int = 0, entryType: Int = 0, vcPass: Boolean = false, ply: Int = 0): Unit = {
    var idx = this.calculateBeginIdx(hashKey)
    val endIdx = idx + BUCKET_SIZE

    var targetIdx = -1
    var targetDepth = Int.MaxValue

    var take = true
    while (idx < endIdx && take) {
      val key = this.keys(idx)
      val entry = new TTEntry(this.entries(idx))

      if (key == 0 && entry.isEmpty) {
        targetIdx = idx
        take = false
      } else if (key == hashKey.raw) {
        if (entry.depth > depth) return

        targetIdx = idx
        take = false
      } else {
        if (targetDepth > entry.depth) {
          targetIdx = idx
          targetDepth = entry.depth
        }

        idx += 1
      }
    }

    this.keys(targetIdx) = hashKey.raw
    this.entries(targetIdx) = TTEntry(eval, score, bestMove, depth, entryType, vcPass).raw
  }

  def find(hashKey: HashKey, ply: Int = 0): TTEntry = {
    var idx = this.calculateBeginIdx(hashKey)
    val endIdx = idx + BUCKET_SIZE

    while (idx < endIdx) {
      if (this.keys(idx) == hashKey.raw)
        return new TTEntry(this.entries(idx))

      idx += 1
    }

    TTEntry.empty
  }

  def resize(tableSize: Int): Unit = {
    if (this.tableSize == tableSize) return
  }

  def clear(): Unit = {
  }

  private def calculateBeginIdx(hashKey: HashKey): Int =
    (hashKey.raw % this.tableSize).toInt.abs

}

object TranspositionTable {

  private val BUCKET_SIZE: Int = 4

  def ofSizeKb(sizeInKb: Int): TranspositionTable =
    new TranspositionTable((sizeInKb * 64) / BUCKET_SIZE)

  def empty: TranspositionTable = TranspositionTable.ofSizeKb(128 * 1024)

}
