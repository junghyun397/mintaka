package engine.cache

import renju.Board
import renju.hash.HashKey
import renju.notation.Flag
import renju.util.LRUCache

import java.util.Collections.synchronizedMap

class LRUMemo(
  private val blackMemo: java.util.Map[Long, Float] = java.util.concurrent.ConcurrentHashMap(),
  private val whiteMemo: java.util.Map[Long, Float] = java.util.concurrent.ConcurrentHashMap(),

  private val blackCache: java.util.Map[Long, Float] = synchronizedMap(LRUCache[Long, Float](1_000)),
  private val whiteCache: java.util.Map[Long, Float] = synchronizedMap(LRUCache[Long, Float](1_000)),
) extends Cloneable {

  def probe(key: HashKey, color: Byte): Option[Float] =
    if (color == Flag.BLACK)
      this.probeBlack(key.raw)
    else
      this.probeWhite(key.raw)

  def probe(board: Board): Option[Float] =
    if (board.isNextColorBlack)
      this.probeBlack(board.hashKey.raw)
    else
      this.probeWhite(board.hashKey.raw)

  def probe(board: Board, move: Int): Option[Float] = {
    val key = board.hashKey.move(move, board.nextColorFlag.raw).raw
    if (board.isNextColorBlack)
      this.probeBlack(key)
    else
      this.probeWhite(key)
  }

  private def probeBlack(key: Long): Option[Float] =
    Option(this.blackCache.get(key))
      .orElse(Option(this.blackMemo.get(key)))

  private def probeWhite(key: Long): Option[Float] =
    Option(this.whiteCache.get(key))
      .orElse(Option(this.whiteMemo.get(key)))

  def write(key: Long, color: Byte, eval: Float): Unit =
    if (color == Flag.BLACK)
      this.writeBlack(key, eval)
    else
      this.writeWhite(key, eval)

  def write(board: Board, eval: Float): Unit =
    if (board.isNextColorBlack)
      this.writeBlack(board.hashKey.raw, eval)
    else
      this.writeWhite(board.hashKey.raw, eval)

  def write(board: Board, move: Int, eval: Float): Unit = {
    val key = board.hashKey.move(move, board.nextColorFlag.raw)
    if (board.isNextColorBlack)
      this.writeBlack(key.raw, eval)
    else
      this.writeWhite(key.raw, eval)
  }

  private def writeBlack(key: Long, eval: Float): Unit = {
    this.blackCache.put(key, eval)
    this.blackMemo.put(key, eval)
  }

  private def writeWhite(key: Long, eval: Float): Unit = {
    this.whiteCache.put(key, eval)
    this.whiteMemo.put(key, eval)
  }

  override def clone(): LRUMemo = LRUMemo(this.blackCache, this.whiteCache)

}

object LRUMemo {

  def empty: LRUMemo = LRUMemo()

}
