
package engine.util

import org.bson.codecs.{BsonDocumentCodec, DecoderContext, EncoderContext}
import org.bson.io.BasicOutputBuffer
import org.bson.{BsonBinaryReader, BsonBinaryWriter, BsonDocument}

import java.nio.ByteBuffer
import scala.language.implicitConversions

extension (d: BsonDocument) {
  
  def binary: Array[Byte] = {
    val outputBuffer = BasicOutputBuffer()
    val writer = BsonBinaryWriter(outputBuffer)
    BsonDocumentCodec().encode(writer, d, EncoderContext.builder.isEncodingCollectibleDocument(true).build)

    outputBuffer.toByteArray
  }
  
}

def bsonFromBinary(binary: Array[Byte]): Option[BsonDocument] =
  Option(BsonDocumentCodec().decode(BsonBinaryReader(ByteBuffer.wrap(binary)), DecoderContext.builder.build))
