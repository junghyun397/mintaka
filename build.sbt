import sbt.Keys.version

val scala2Version = "2.13.8"
val scala3Version = "3.1.1"

ThisBuild / version := "0.1.0-SNAPSHOT"

ThisBuild / libraryDependencies ++= Seq(
  "org.scalatest" %% "scalatest" % "3.2.12" % Test
)

lazy val utils = (project in file("util"))
  .settings(
    name := "utils",
    description := "utility library",

    scalaVersion := scala2Version,
    publishMavenStyle := true,
  )

lazy val jrenju = (project in file("rule"))
  .settings(
    name := "jrenju",
    description := "jrenju jvm renju library written in scala",

    scalaVersion := scala2Version,
    publishMavenStyle := true,
  )
  .dependsOn(utils)

lazy val protobuf = (project in file("protobuf"))
  .settings(
    scalaVersion := scala3Version
  )
  .enablePlugins(Fs2Grpc)

lazy val app = (project in file("app"))
  .settings(
    name := "app",
    description := "online self-learning renju solver",

    scalaVersion := scala3Version,

    libraryDependencies ++= Seq(
      "org.typelevel" %% "cats-core" % "2.8.0",
      "org.typelevel" %% "cats-effect" % "3.3.14",
      "org.typelevel" %% "munit-cats-effect-3" % "1.0.7" % Test,

      "co.fs2" %% "fs2-core" % "3.2.14",
      "co.fs2" %% "fs2-reactive-streams" % "3.2.14",

      "org.typelevel" %% "log4cats-core" % "2.4.0",
      "org.typelevel" %% "log4cats-slf4j" % "2.4.0",

      "org.reactivemongo" %% "reactivemongo" % "1.1.0-RC3",

      "io.grpc" % "grpc-netty-shaded" % scalapb.compiler.Version.grpcJavaVersion,

      "org.deeplearning4j" % "deeplearning4j-core" % "1.0.0-M1.1",
      "org.nd4j" % "nd4j-native-platform" % "1.0.0-M1.1",

      "org.yaml" % "snakeyaml" % "1.30",
    ),
  )
  .dependsOn(utils, jrenju, protobuf)

lazy val root = (project in file("."))
  .settings(
    name := "Kvine",
  )
  .aggregate(utils, jrenju, protobuf, app)
