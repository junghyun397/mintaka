package renju.hash

import renju.hash.HashKey.table
import renju.notation.{Flag, Renju}

import scala.util.Random

// Zobrist Hash, collision at sqrt(2^64) = 2^32
class HashKey(val raw: Long) extends AnyVal {

  def move(move: Int, flag: Byte): HashKey = {
    val raw =
      if (flag == Flag.BLACK) this.raw ^ table(move)
      else this.raw ^ table(Renju.BOARD_SIZE + move)

    new HashKey(raw)
  }

}

object HashKey {

  private val TABLE_SEED = 42

  private val table: Array[Long] = {
    val random = new Random(TABLE_SEED)
    Array.fill[Long](Renju.BOARD_SIZE * 2)(random.nextLong())
  }

  val empty: HashKey = new HashKey(new Random(TABLE_SEED << 1).nextLong())

  def hash(field: Array[Byte]): HashKey = {
    var result = this.empty.raw

    var flag = Flag.WALL
    var move = 0
    while (move < Renju.BOARD_SIZE) {
      flag = field(move)

      if (flag == Flag.BLACK)
        result ^= this.table(move)
      else if (flag == Flag.WHITE)
        result ^= this.table(Renju.BOARD_SIZE + move)
        
      move += 1
    }

    new HashKey(result)
  }

}
