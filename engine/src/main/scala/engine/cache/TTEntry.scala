package engine.cache

// jvm long(8bytes)
// eval   score  bestMove depth entry-type margin vc-pass is-empty
// 16bits 16bits 8bits    8bits 2bits      5bits  1bit    1bit
// 48<    32<    24<      16<   8<         6<     1<      0<
class TTEntry(val raw: Long) extends AnyVal {

  def isEmpty: Boolean = this.raw == 0

  def nonEmpty: Boolean = this.raw != 0

  def eval: Int = (this.raw >>> 48).toInt

  def score: Int = (this.raw >>> 32).toInt & 0x0000ffff

  def bestMove: Int = (this.raw >>> 24).toInt & 0x000000ff

  def depth: Int = (this.raw >>> 16).toInt & 0x000000ff

  def entryType: Int = (this.raw >>> 8).toInt & 0x00000003

  def vcPass: Boolean = ((this.raw >>> 1).toInt & 0x00000002) == 0x02

}

object TTEntry {

  def apply(eval: Int, score: Int, bestMove: Int, depth: Int, entryType: Int, vcPass: Boolean): TTEntry =
    new TTEntry(
      0x01 |
        (eval << 48) |
        (score << 32) |
        (bestMove << 24) |
        (depth << 16) |
        (entryType << 8) |
        (if (vcPass) 0x02 else 0x00)
    )

  val empty = new TTEntry(0x00)

}
