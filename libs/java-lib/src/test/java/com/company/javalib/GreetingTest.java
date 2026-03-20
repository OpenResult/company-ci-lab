package com.company.javalib;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class GreetingTest {
    @Test
    void greetsByName() {
        assertEquals("hello, codex", Greeting.greet("codex"));
    }
}
