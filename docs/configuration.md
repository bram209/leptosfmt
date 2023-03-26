# Configuration

## attr_value_brace_style

Wether or not to add braces around single expressions for attribute values.

- **Default value:** "AlwaysUnlessLit" (Up for discussion)
- **Possible values:** "Always", "AlwaysUnlessLit", "WhenRequired", "Preserve"

### Examples

`"AlwaysUnlessLiteral"` (default):

```rust
<div on:click=move |_| set_value(0) disabled=is_disabled />
<img width=100 height={200} class="banner" alt={"test"} />

// BECOMES

<div on:click={move |_| set_value(0)} disabled={is_disabled} />
<img width=100 height=200 class="banner" alt="test" />
```

`"Always"`:

```rust
<div on:click=move |_| set_value(0) disabled=is_disabled />
<img width=100 height={200} class="banner" alt={"test"}>

// BECOMES

<div on:click={move |_| set_value(0)} disabled={is_disabled} />
<img width={100} height={200} class={"banner"} alt={"test"} />
```

`"WhenRequired"`:

```rust
<div on:click={move |_| set_value(0)} disabled={is_disabled} />
<img width=100 height={200} class="banner" alt={"test"} />

// BECOMES

<div on:click=move |_| set_value(0) disabled=is_disabled />
<img width=100 height=200 class="banner" alt="test" />
```

`"Preserve"`:

```rust
<div on:click=move |_| set_value(0) />                              // stays untouched
<div on:click={move |_| set_value(0)} />                            // stays untouched
<img width=100 height={200} class="banner" src={"./banner.jpg"} />    // stays untouched

```
