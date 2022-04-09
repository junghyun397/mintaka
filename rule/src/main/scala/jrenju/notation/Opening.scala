package jrenju.notation

sealed abstract class Opening(val name: String, val jName: String, val rotation: Rotation.Value) {

  class I1(rotation: Rotation.Value) extends Opening("I1", "Chosei", rotation)
  class I2(rotation: Rotation.Value) extends Opening("I2", "Kyogetsu", rotation)
  class I3(rotation: Rotation.Value) extends Opening("I3", "Kosei", rotation)
  class I4(rotation: Rotation.Value) extends Opening("I4", "Suigetsu", rotation)
  class I5(rotation: Rotation.Value) extends Opening("I5", "Ryusei", rotation)
  class I6(rotation: Rotation.Value) extends Opening("I6", "Ungetsu", rotation)
  class I7(rotation: Rotation.Value) extends Opening("I7", "Hogetsu", rotation)
  class I8(rotation: Rotation.Value) extends Opening("I8", "Rangetsu", rotation)
  class I9(rotation: Rotation.Value) extends Opening("I9", "Gingetsu", rotation)
  class I10(rotation: Rotation.Value) extends Opening("I10", "Myojo", rotation)
  class I11(rotation: Rotation.Value) extends Opening("I11", "Shagetsu", rotation)
  class I12(rotation: Rotation.Value) extends Opening("I12", "Meigetsu", rotation)
  class I13(rotation: Rotation.Value) extends Opening("I13", "Suisei", rotation)

  class D1(rotation: Rotation.Value) extends Opening("D1", "Kansei", rotation)
  class D2(rotation: Rotation.Value) extends Opening("D2", "Keigetsu", rotation)
  class D3(rotation: Rotation.Value) extends Opening("D3", "Sosei", rotation)
  class D4(rotation: Rotation.Value) extends Opening("D4", "Kagetsu", rotation)
  class D5(rotation: Rotation.Value) extends Opening("D5", "Zangetsu", rotation)
  class D6(rotation: Rotation.Value) extends Opening("D6", "Ugetsu", rotation)
  class D7(rotation: Rotation.Value) extends Opening("D7", "Kinsei", rotation)
  class D8(rotation: Rotation.Value) extends Opening("D8", "Shogetsu", rotation)
  class D9(rotation: Rotation.Value) extends Opening("D9", "Kyugetsu", rotation)
  class D10(rotation: Rotation.Value) extends Opening("D10", "Shingetsu", rotation)
  class D11(rotation: Rotation.Value) extends Opening("D11", "Zuisei", rotation)
  class D12(rotation: Rotation.Value) extends Opening("D12", "Sangetsu", rotation)
  class D13(rotation: Rotation.Value) extends Opening("D13", "Yusei", rotation)

}

object Opening {

  def detect(boardField: Array[Byte], latestMove: Int): Option[Opening] = Option.empty

}
