package utils.lang

import org.bson.codecs.{BsonDocumentCodec, DecoderContext, EncoderContext}
import org.bson.io.BasicOutputBuffer
import org.bson.{BsonBinaryReader, BsonBinaryWriter, BsonDocument}

import java.nio.ByteBuffer
import scala.language.implicitConversions

class BsonDocumentOps(private val x: BsonDocument) extends AnyVal {

  def binary: Array[Byte] = {
    val outputBuffer = new BasicOutputBuffer
    val writer = new BsonBinaryWriter(outputBuffer)
    new BsonDocumentCodec().encode(writer, x, EncoderContext.builder.isEncodingCollectibleDocument(true).build)

    outputBuffer.toByteArray
  }

}

object BsonDocumentOps {

  implicit def bsonDocumentOps(document: BsonDocument): BsonDocumentOps = new BsonDocumentOps(document)

  def fromBinary(binary: Array[Byte]): BsonDocument =
    new BsonDocumentCodec().decode(new BsonBinaryReader(ByteBuffer.wrap(binary)), DecoderContext.builder.build)

}
