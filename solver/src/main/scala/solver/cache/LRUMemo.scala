package solver.cache

import jrenju.Board
import jrenju.ZobristHash.IncrementHash
import jrenju.notation.Flag
import utils.lang.LRUCache

import java.util.Collections.synchronizedMap

class LRUMemo(
  private val blackMemo: java.util.Map[Long, Float] = java.util.concurrent.ConcurrentHashMap(),
  private val whiteMemo: java.util.Map[Long, Float] = java.util.concurrent.ConcurrentHashMap(),

  private val blackCache: java.util.Map[Long, Float] = synchronizedMap(LRUCache[Long, Float](1_000)),
  private val whiteCache: java.util.Map[Long, Float] = synchronizedMap(LRUCache[Long, Float](1_000)),
) extends Cloneable {

  def probe(key: Long, color: Byte): Option[Float] =
    if (color == Flag.BLACK)
      this.probeBlack(key)
    else
      this.probeWhite(key)

  def probe(board: Board): Option[Float] =
    if (board.isNextColorBlack)
      this.probeBlack(board.hashKey)
    else
      this.probeWhite(board.hashKey)

  def probe(board: Board, move: Int): Option[Float] = {
    val key = board.hashKey.incrementBoardHash(move, board.nextColorFlag.raw)
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
      this.writeBlack(board.hashKey, eval)
    else
      this.writeWhite(board.hashKey, eval)

  def write(board: Board, move: Int, eval: Float): Unit = {
    val key = board.hashKey.incrementBoardHash(move, board.nextColorFlag.raw)
    if (board.isNextColorBlack)
      this.writeBlack(key, eval)
    else
      this.writeWhite(key, eval)
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
