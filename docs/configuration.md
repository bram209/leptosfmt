# Configuration

## attr_value_brace_style

Wether or not to add braces around single expressions for attribute values.

**Default value:** "Always"
**Possible values:** "Preserve", "Always", "WhenRequired"

### Examples

`"Preserve"`:

```rust
<div on:click=move |_| set_value(0) />      // stays untouched
<div on:click={move |_| set_value(0)} />    // stays untouched
```

`"Always"`:

```rust
<div on:click=move |_| set_value(0) />
<div on:click={move |_| set_value(0)} />
```

becomes

```rust
<div on:click={move |_| set_value(0)} />
<div on:click={move |_| set_value(0)} />
```

`"WhenRequired"`:

```rust
<div on:click={move |_| set_value(0)} />

<div class={
  let foo = "foo";
  let bar = "bar";
  foo + bar
} />
```

becomes

```rust
<div on:click=move |_| set_value(0) />

<div class={
  let foo = "foo";
  let bar = "bar";
  foo + bar
} />
```
