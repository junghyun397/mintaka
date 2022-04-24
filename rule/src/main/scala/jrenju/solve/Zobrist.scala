package jrenju.solve

import jrenju.notation.{Flag, Renju}

import scala.util.Random

object Zobrist {

  private val TABLE_SEED = 10204

  private val table: Array[Long] = {
    val random = new Random(TABLE_SEED)
    Array.fill[Long](Renju.BOARD_LENGTH * 2)(random.nextLong())
  }

  private val stripTable: Array[Long] = {
    val random = new Random(TABLE_SEED)
    Array.fill[Long](Renju.BOARD_WIDTH * 3)(random.nextLong())
  }

  val empty: Long = new Random(TABLE_SEED << 1).nextLong()

  def hash(field: Array[Byte]): Long = {
    var result = this.empty

    var flag = Flag.WALL
    for (move <- 0 until Renju.BOARD_LENGTH) {
      flag = field(move)
      if (Flag.isExist(flag)) {
        if (flag == Flag.BLACK)
          result ^= this.table(move)
        else
          result ^= this.table(Renju.BOARD_LENGTH + move)
      }
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

    def incrementHash(move: Int, isBlack: Boolean): Long =
      if (isBlack) source ^ table(move)
      else source ^ table(Renju.BOARD_LENGTH + move)

  }

}
