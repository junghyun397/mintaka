package jrenju.solve

import jrenju.Board
import jrenju.notation.Flag
import jrenju.ZobristHash.IncrementHash
import utils.lang.LRUCache

import java.util.Collections.synchronizedMap

class LRUMemo(
  private val blackMemo: java.util.Map[Long, Float] = new java.util.concurrent.ConcurrentHashMap(),
  private val whiteMemo: java.util.Map[Long, Float] = new java.util.concurrent.ConcurrentHashMap(),

  private val blackCache: java.util.Map[Long, Float] = synchronizedMap(new LRUCache[Long, Float](1_000)),
  private val whiteCache: java.util.Map[Long, Float] = synchronizedMap(new LRUCache[Long, Float](1_000)),
) extends Cloneable {

  def probe(key: Long, color: Byte): Option[Float] =
    if (color == Flag.BLACK)
      this.probeBlack(key)
    else
      this.probeWhite(key)

  def probe(board: Board): Option[Float] =
    if (board.isNextColorBlack)
      this.probeBlack(board.zobristKey)
    else
      this.probeWhite(board.zobristKey)

  def probe(board: Board, move: Int): Option[Float] = {
    val key = board.zobristKey.incrementHash(move, board.nextColorFlag)
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
      this.writeBlack(board.zobristKey, eval)
    else
      this.writeWhite(board.zobristKey, eval)

  def write(board: Board, move: Int, eval: Float): Unit = {
    val key = board.zobristKey.incrementHash(move, board.nextColorFlag)
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

  override def clone(): LRUMemo = new LRUMemo(this.blackCache, this.whiteCache)

}

object LRUMemo {

  def empty: LRUMemo = new LRUMemo()

}
