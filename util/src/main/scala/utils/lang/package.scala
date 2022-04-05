package utils

package object lang {

  implicit class IntTransform(val value: Int) {

    def dotIfZero: String = if (value == 0) "." else value.toString

  }


}
