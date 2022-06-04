package jrenju.notation

import jrenju.ZobristHash
import jrenju.ZobristHash.IncrementHash

sealed abstract class Opening(val name: String, val jName: String, val rotation: Rotation.Value)

object Opening {

  case class D1(override val rotation: Rotation.Value) extends Opening("D1", "Kansei", rotation)
  case class D2(override val rotation: Rotation.Value) extends Opening("D2", "Keigetsu", rotation)
  case class D3(override val rotation: Rotation.Value) extends Opening("D3", "Sosei", rotation)
  case class D4(override val rotation: Rotation.Value) extends Opening("D4", "Kagetsu", rotation)
  case class D5(override val rotation: Rotation.Value) extends Opening("D5", "Zangetsu", rotation)
  case class D6(override val rotation: Rotation.Value) extends Opening("D6", "Ugetsu", rotation)
  case class D7(override val rotation: Rotation.Value) extends Opening("D7", "Kinsei", rotation)
  case class D8(override val rotation: Rotation.Value) extends Opening("D8", "Shogetsu", rotation)
  case class D9(override val rotation: Rotation.Value) extends Opening("D9", "Kyugetsu", rotation)
  case class D10(override val rotation: Rotation.Value) extends Opening("D10", "Shingetsu", rotation)
  case class D11(override val rotation: Rotation.Value) extends Opening("D11", "Zuisei", rotation)
  case class D12(override val rotation: Rotation.Value) extends Opening("D12", "Sangetsu", rotation)
  case class D13(override val rotation: Rotation.Value) extends Opening("D13", "Yusei", rotation)

  case class I1(override val rotation: Rotation.Value) extends Opening("I1", "Chosei", rotation)
  case class I2(override val rotation: Rotation.Value) extends Opening("I2", "Kyogetsu", rotation)
  case class I3(override val rotation: Rotation.Value) extends Opening("I3", "Kosei", rotation)
  case class I4(override val rotation: Rotation.Value) extends Opening("I4", "Suigetsu", rotation)
  case class I5(override val rotation: Rotation.Value) extends Opening("I5", "Ryusei", rotation)
  case class I6(override val rotation: Rotation.Value) extends Opening("I6", "Ungetsu", rotation)
  case class I7(override val rotation: Rotation.Value) extends Opening("I7", "Hogetsu", rotation)
  case class I8(override val rotation: Rotation.Value) extends Opening("I8", "Rangetsu", rotation)
  case class I9(override val rotation: Rotation.Value) extends Opening("I9", "Gingetsu", rotation)
  case class I10(override val rotation: Rotation.Value) extends Opening("I10", "Myojo", rotation)
  case class I11(override val rotation: Rotation.Value) extends Opening("I11", "Shagetsu", rotation)
  case class I12(override val rotation: Rotation.Value) extends Opening("I12", "Meigetsu", rotation)
  case class I13(override val rotation: Rotation.Value) extends Opening("I13", "Suisei", rotation)

  private val directHashes: Map[Rotation.Value, Long] = Map(
    Rotation.STRAIGHT -> ZobristHash.empty
      .incrementBoardHash(112, Flag.BLACK)
      .incrementBoardHash(127, Flag.WHITE),
    Rotation.CLOCKWISE -> ZobristHash.empty
      .incrementBoardHash(112, Flag.BLACK)
      .incrementBoardHash(113, Flag.WHITE),
    Rotation.COUNTER_CLOCKWISE -> ZobristHash.empty
      .incrementBoardHash(112, Flag.BLACK)
      .incrementBoardHash(111, Flag.WHITE),
    Rotation.OVERTURN -> ZobristHash.empty
      .incrementBoardHash(112, Flag.BLACK)
      .incrementBoardHash(97, Flag.WHITE)
  )

  private val directOpeningHashes: Map[Int, Rotation.Value => Opening] = Map(
    0 -> { r => Opening.D1(r) },
    1 -> { r => Opening.D2(r) },
    2 -> { r => Opening.D3(r) },
    3 -> { r => Opening.D4(r) },
    4 -> { r => Opening.D5(r) },
    5 -> { r => Opening.D6(r) },
    6 -> { r => Opening.D7(r) },
    7 -> { r => Opening.D8(r) },
    8 -> { r => Opening.D9(r) },
    9 -> { r => Opening.D10(r) },
    10 -> { r => Opening.D11(r) },
    11 -> { r => Opening.D12(r) },
    12 -> { r => Opening.D13(r) },
  )

  private val indirectHashes: Map[Rotation.Value, Long] = Map(
    Rotation.STRAIGHT -> ZobristHash.empty
      .incrementBoardHash(112, Flag.BLACK)
      .incrementBoardHash(128, Flag.WHITE),
    Rotation.CLOCKWISE -> ZobristHash.empty
      .incrementBoardHash(112, Flag.BLACK)
      .incrementBoardHash(98, Flag.WHITE),
    Rotation.COUNTER_CLOCKWISE -> ZobristHash.empty
      .incrementBoardHash(112, Flag.BLACK)
      .incrementBoardHash(126, Flag.WHITE),
    Rotation.OVERTURN -> ZobristHash.empty
      .incrementBoardHash(112, Flag.BLACK)
      .incrementBoardHash(96, Flag.WHITE)
  )

  private val indirectOpeningHashes: Map[Int, Rotation.Value => Opening] = Map(
    0 -> { r => Opening.I1(r) },
    1 -> { r => Opening.I2(r) },
    2 -> { r => Opening.I3(r) },
    3 -> { r => Opening.I4(r) },
    4 -> { r => Opening.I5(r) },
    5 -> { r => Opening.I6(r) },
    6 -> { r => Opening.I7(r) },
    7 -> { r => Opening.I8(r) },
    8 -> { r => Opening.I9(r) },
    9 -> { r => Opening.I10(r) },
    10 -> { r => Opening.I11(r) },
    11 -> { r => Opening.I12(r) },
    12 -> { r => Opening.I13(r) },
  )

  def detect(boardField: Array[Byte], latestMove: Int): Option[Opening] = Option.empty

}
