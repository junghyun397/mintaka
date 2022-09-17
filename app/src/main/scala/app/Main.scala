package app

import cats.effect.{IO, IOApp}
import cats.{Eq, Show}
import jrenju.Board

object Main extends IOApp.Simple {

  override val run: IO[Unit] = for {
    _ <- IO.println("hello, scala!")
    _ <- IO.println("line two here")
  } yield ()

}
