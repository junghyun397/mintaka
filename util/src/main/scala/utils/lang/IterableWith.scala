package utils.lang

import scala.collection.AbstractIterator

trait IterableWith[T] extends Iterable[T] {

  def maxSize: Int

  def elementAt(idx: Int): T

  val iterator: Iterator[T] = new AbstractIterator[T] { self =>

    private var idx: Int = 0

    override def hasNext: Boolean = idx < maxSize

    override def next(): T = {
      val status = elementAt(idx)

      self.idx += 1

      status
    }

  }

}
