#!/usr/bin/env bash
set -euo pipefail

test -f pom.xml
test -f src/main/java/com/company/springapi/SpringApiApplication.java
test -f src/test/java/com/company/springapi/HealthIntegrationTest.java
echo "spring-api scaffold verification passed"
