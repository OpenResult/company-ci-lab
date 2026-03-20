package com.company.javalib;

public final class Greeting {
    private Greeting() {
    }

    public static String greet(String name) {
        return "hello, " + name;
    }
}
