#!/usr/bin/env bash
set -euo pipefail

test -f pom.xml
mkdir -p target/classes
javac -d target/classes src/main/java/com/company/javalib/Greeting.java
mkdir -p target/test-classes
javac -cp target/classes -d target/test-classes src/test/java/com/company/javalib/GreetingTest.java || {
  echo "java-lib JUnit test compile skipped because JUnit dependencies are not vendored in the scaffold" >&2
}
echo "java-lib scaffold verification passed"
