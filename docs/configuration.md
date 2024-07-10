# Configuration

## attr_value_brace_style

Whether or not to add braces around single expression attribute values.

- **Default value:** "WhenRequired"
- **Possible values:** "Always", "AlwaysUnlessLit", "WhenRequired", "Preserve"

### Examples

`"AlwaysUnlessLit"`:

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

`"WhenRequired"` (default):

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
<img width=100 height={200} class="banner" src={"./banner.jpg"} />  // stays untouched

```
