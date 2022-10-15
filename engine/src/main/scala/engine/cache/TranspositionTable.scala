package engine.cache

import engine.cache.TranspositionTable.BUCKET_SIZE
import renju.hash.HashKey

// Transposition Table with Zobrist Hash, collision at
// key: Long = zobrist key 64bits
// entry: Long = bestMove 8bits + eval 16bits + score 16bits + depth 8bits + type 2bits
// 1 key-entry pair = 128bits, 16bytes 1KB = 64 pairs
class TranspositionTable(
  val tableSize: Int
) {

  private val totalSize: Int = this.tableSize * BUCKET_SIZE

  private val keys: Array[Long] = Array[Long](this.totalSize)
  private val entries: Array[Long] = Array[Long](this.totalSize)

  def write(hashKey: HashKey, bestMove: Int = 0, eval: Int, score: Int = 0, depth: Int = 0, entryType: Int = 0, ply: Int = 0): Unit = {
    val beginIdx: Int = this.calculateBeginIdx(hashKey)
    val endIdx: Int = beginIdx + BUCKET_SIZE

    var targetIdx: Int = -1

    var idx: Int = beginIdx
    var escape: Boolean = false
    while (idx < endIdx && !escape) {
      val key = this.keys(idx)
      val entry = this.entries(idx)

      if (key == 0 && entry == 0) {
        targetIdx = idx
        escape = true
      } else if (key == hashKey.raw) {
        targetIdx = idx
        escape = true
      } else {
        idx += 1
      }
    }

    this.keys(idx) = hashKey.raw
    this.entries(idx) = eval
  }

  def find(hashKey: HashKey, ply: Int = 0): Long = {
    val beginIdx: Int = this.calculateBeginIdx(hashKey)
    val endIdx: Int = beginIdx + BUCKET_SIZE

    var idx: Int = beginIdx
    while (idx < endIdx) {
      val key = this.keys(idx)

      if (key == hashKey.raw)
        return this.entries(idx)

      idx += 1
    }

    0
  }

  private def calculateBeginIdx(hashKey: HashKey): Int =
    (hashKey.raw % this.totalSize).toInt

}

object TranspositionTable {

  private val BUCKET_SIZE: Int = 4

  def extractBestMove(entry: Long): Int = ???

  def extractBestEval(entry: Long): Int = ???

  def extractBestScore(entry: Long): Int = ???

  def extractBestDepth(entry: Long): Int = ???

  def extractBestEntryType(entry: Long): Int = ???

}
