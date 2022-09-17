package jrenju

import jrenju.notation.{Flag, Renju}

import scala.util.Random

object ZobristHash {

  private val TABLE_SEED = 42

  private val table: Array[Long] = {
    val random = new Random(TABLE_SEED)
    Array.fill[Long](Renju.BOARD_SIZE * 2)(random.nextLong())
  }

  private val stripTable: Array[Long] = {
    val random = new Random(TABLE_SEED)
    Array.fill[Long](Renju.BOARD_WIDTH * 3)(random.nextLong())
  }

  val empty: Long = new Random(TABLE_SEED << 1).nextLong()

  def boardHash(field: Array[Byte]): Long = {
    var result = this.empty

    var flag = Flag.WALL
    for (move <- 0 until Renju.BOARD_SIZE) {
      flag = field(move)
      if (flag == Flag.BLACK)
        result ^= this.table(move)
      else if (flag == Flag.WHITE)
        result ^= this.table(Renju.BOARD_SIZE + move)
    }

    result
  }

  def stripHash(field: Array[Byte]): Long = {
    var result = this.empty

    var flag = Flag.WALL
    for (move <- field.indices) {
      flag = field(move)
      if (flag == Flag.BLACK)
        result ^= this.stripTable(move)
      else if (flag == Flag.WHITE)
        result ^= this.stripTable(Renju.BOARD_WIDTH + move)
    }

    if (field.length != Renju.BOARD_WIDTH)
      for (move <- field.length until Renju.BOARD_WIDTH)
        result ^= this.stripTable(Renju.BOARD_WIDTH * 2 + move)

    result
  }

  implicit class IncrementHash(source: Long) {

    def incrementBoardHash(move: Int, flag: Byte): Long =
      if (flag == Flag.BLACK) source ^ table(move)
      else source ^ table(Renju.BOARD_SIZE + move)

  }

}
