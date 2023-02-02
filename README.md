# Really Simple Communication

This is a really simple serial comunication terminal application
Much like picocom.

## Usage ##

Press C-a C-h for a list of commands

## Macros ##

It is possible to define up to 10 macros in a yaml file.
This file must be called "macros.rscom" and be located in the current working directory.
The contents could look like this:

```yaml
macros:
    - name: Macro 0
      content: Foo
    - name: Macro 1
      content: Bar
    - name: Macro 2
      content: AnotherMacro
    - name: Macro 3
      content: OneMore
```