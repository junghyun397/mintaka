package jrenju.solve

import jrenju.Board
import jrenju.notation.Flag
import jrenju.solve.ZobristHash.IncrementHash
import utils.lang.LRUCache

class LRUMemo(
  private val black: java.util.Map[Long, Float] = LRUCache[Long, Float](100_000),
  private val white: java.util.Map[Long, Float] = LRUCache[Long, Float](100_000),
) extends Cloneable {

  def probe(board: Board): Option[Float] =
    if (board.colorRaw == Flag.BLACK)
      Option(this.black.get(board.zobristKey))
    else
      Option(this.white.get(board.zobristKey))

  def probe(board: Board, move: Int, flag: Byte): Option[Float] =
    if (flag == Flag.BLACK)
      Option(this.black.get(board.zobristKey.incrementHash(move, flag)))
    else
      Option(this.white.get(board.zobristKey.incrementHash(move, flag)))

  def write(board: Board, eval: Float): Unit =
    if (board.colorRaw == Flag.BLACK)
      this.black.put(board.zobristKey, eval)
    else
      this.white.put(board.zobristKey, eval)

  def write(board: Board, move: Int, flag: Byte, eval: Float): Unit =
    if (flag == Flag.BLACK)
      this.black.put(board.zobristKey.incrementHash(move, flag), eval)
    else
      this.white.put(board.zobristKey.incrementHash(move, flag), eval)

  override def clone(): LRUMemo = new LRUMemo(this.black, this.white)

}
