package utils.math

object FNV1a {

  private val INIT32 = BigInt("811c9dc5", 16)
  private val INIT64 = BigInt("cbf29ce484222325", 16)
  private val PRIME32 = BigInt("01000193", 16)
  private val PRIME64 = BigInt("100000001b3", 16)
  private val MOD32 = BigInt("2").pow(32)
  private val MOD64 = BigInt("2").pow(64)
  private val MASK = 0xff

  @inline private final def calc(prime: BigInt, mod: BigInt)(hash: BigInt, b: Byte): BigInt = ((hash * prime) % mod) ^ (b & MASK)

  @inline private final def calcA(prime: BigInt, mod: BigInt)(hash: BigInt, b: Byte): BigInt = ((hash ^ (b & MASK)) * prime) % mod

  @inline final def hash32a(data: Array[Byte]): BigInt = data.foldLeft(INIT32)(calcA(PRIME32, MOD32))

  @inline final def hash64a(data: Array[Byte]): BigInt = data.foldLeft(INIT64)(calcA(PRIME64, MOD64))

}
