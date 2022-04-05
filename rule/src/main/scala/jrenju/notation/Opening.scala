package jrenju.notation

import jrenju.Board

sealed abstract class Opening(val name: String, val rotation: Rotation.Value) {

  class I1(rotation: Rotation.Value) extends Opening("Chosei", rotation)
  class I2(rotation: Rotation.Value) extends Opening("Kyogetsu", rotation)
  class I3(rotation: Rotation.Value) extends Opening("Kosei", rotation)
  class I4(rotation: Rotation.Value) extends Opening("Suigetsu", rotation)
  class I5(rotation: Rotation.Value) extends Opening("Ryusei", rotation)
  class I6(rotation: Rotation.Value) extends Opening("Ungetsu", rotation)
  class I7(rotation: Rotation.Value) extends Opening("Hogetsu", rotation)
  class I8(rotation: Rotation.Value) extends Opening("Rangetsu", rotation)
  class I9(rotation: Rotation.Value) extends Opening("Gingetsu", rotation)
  class I10(rotation: Rotation.Value) extends Opening("Myojo", rotation)
  class I11(rotation: Rotation.Value) extends Opening("Shagetsu", rotation)
  class I12(rotation: Rotation.Value) extends Opening("Meigetsu", rotation)
  class I13(rotation: Rotation.Value) extends Opening("Suisei", rotation)

  class D1(rotation: Rotation.Value) extends Opening("Kansei", rotation)
  class D2(rotation: Rotation.Value) extends Opening("Keigetsu", rotation)
  class D3(rotation: Rotation.Value) extends Opening("Sosei", rotation)
  class D4(rotation: Rotation.Value) extends Opening("Kagetsu", rotation)
  class D5(rotation: Rotation.Value) extends Opening("Zangetsu", rotation)
  class D6(rotation: Rotation.Value) extends Opening("Ugetsu", rotation)
  class D7(rotation: Rotation.Value) extends Opening("Kinsei", rotation)
  class D8(rotation: Rotation.Value) extends Opening("Shogetsu", rotation)
  class D9(rotation: Rotation.Value) extends Opening("Kyugetsu", rotation)
  class D10(rotation: Rotation.Value) extends Opening("Shingetsu", rotation)
  class D11(rotation: Rotation.Value) extends Opening("Zuisei", rotation)
  class D12(rotation: Rotation.Value) extends Opening("Sangetsu", rotation)
  class D13(rotation: Rotation.Value) extends Opening("Yusei", rotation)

}

object Opening {

  def detect(boardField: Array[Byte], latestMove: Int): Option[Opening] = Option.empty

}
