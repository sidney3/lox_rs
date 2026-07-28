# Calling Convention

## Calling

The caller will lay out our stack as follows:

```
  f arg1 arg2 arg3 _
  ^                ^
  base ptr        stack top
```

Notably this implies that `f := stack_top - nargs - 1`

### Function Body

We begin the function body by "catching up" the compile time stack to represent this by binding the function and its args. This has the effect of injecting these args as locals into the function.

### Calling

`f(a1, a2)` gets resolved by by pushing the `f` variable onto the stack, following by pushing the `a1` and `a2` (which are expressions!) onto the stack. So before `callq`, our stack looks like:

```
    ................................f a1 a2 
  ^                 ^                       ^
base ptr         callers stack          stack top
```

We execute `callq` by loading the function `f`, pushing it onto our code stack, and setting the callee stack as follows

```

f a1 a2 ^
^       |-----stack top
base ptr      
```

where, by compile time magic, `[f, a1, a2]` are already "bound" without the runtime having to do anything.


## Returning

When returning, the runtime does more work. We pop from the value stack until stack_top (in the caller) points to callee.base_ptr. We expect that, upon returning, `stack.len() == callee.bp + fn.nargs + 1`.

Consider the example of a callee with an empty stack calling a nullary function. `f` will be at position 0, and thus `stack.len() == 1 == nargs + 1`.
