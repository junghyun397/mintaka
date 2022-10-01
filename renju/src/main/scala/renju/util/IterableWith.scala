package renju.util

import scala.collection.AbstractIterator

trait IterableWith[T] extends Iterable[T] {
  self =>

  def length: Int

  def elementAt(idx: Int): T

  val iterator: Iterator[T] = new AbstractIterator[T] {

    private var idx: Int = 0

    override def hasNext: Boolean = idx < self.length

    override def next(): T = {
      val status = elementAt(idx)

      this.idx += 1

      status
    }

  }

}
