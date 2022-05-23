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

lazy val core = (project in file("core"))
  .settings(
    name := "core",
    description := "core",

    scalaVersion := scala3Version,

    libraryDependencies ++= Seq(
      "org.typelevel" %% "cats-core" % "2.7.0",
      "org.typelevel" %% "cats-effect" % "3.3.11",

      "co.fs2" %% "fs2-core" % "3.2.7",
      "co.fs2" %% "fs2-reactive-streams" % "3.2.7",

      "org.typelevel" %% "log4cats-core" % "2.3.0",
      "org.typelevel" %% "log4cats-slf4j" % "2.3.0",
    )
  )
  .dependsOn(utils, jrenju)

lazy val protobuf = (project in file("protobuf"))
  .settings(
    scalaVersion := scala3Version
  )
  .enablePlugins(Fs2Grpc)

lazy val app = (project in file("app"))
  .settings(
    name := "app",
    description := "m,n,k-game online self-play learning server",

    scalaVersion := scala3Version,

    libraryDependencies ++= Seq(
      "org.reactivemongo" %% "reactivemongo" % "1.1.0-RC3",

      "io.grpc" % "grpc-netty-shaded" % scalapb.compiler.Version.grpcJavaVersion,

      "org.deeplearning4j" % "deeplearning4j-core" % "1.0.0-M1.1",
      "org.nd4j" % "nd4j-native-platform" % "1.0.0-M1.1",

      "org.yaml" % "snakeyaml" % "1.30",
    ),
  )
  .dependsOn(utils, jrenju, core, protobuf)

lazy val root = (project in file("."))
  .settings(
    name := "B3nzene",
  )
  .aggregate(utils, jrenju, core, protobuf, app)
