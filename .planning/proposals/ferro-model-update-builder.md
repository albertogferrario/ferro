# Proposal: Builder Pattern for Model Updates

## Problem

Model `update` methods use positional `Option` parameters that scale poorly and cause real bugs.

### Current API

```rust
pub async fn update(
    &self,
    slug: Option<String>,
    price: Option<Decimal>,
    currency: Option<String>,
    dietary_tags: Option<Option<Json>>,
    allergens: Option<Option<Json>>,
    image_url: Option<Option<String>>,
    is_available: Option<bool>,
    is_featured: Option<bool>,
) -> Result<Self, FrameworkError>
```

### Issues encountered in practice

1. **Positional confusion**: `allergens` (pos 5) vs `image_url` (pos 6) are both `Option<Option<...>>` — easy to swap silently with no compile error.

2. **Fragile call sites**: Every caller passes `None` for untouched fields. Adding a new column breaks all existing call sites.

```rust
// Setting one field requires knowing all 8 positions:
item.update(None, None, None, None, None, None, None, Some(true)).await?;
```

3. **`Option<Option<T>>` semantics are non-obvious**: `None` = don't touch, `Some(None)` = set NULL, `Some(Some(v))` = set value. This works logically but reads poorly at call sites.

4. **Stale model risk**: Sequential updates on the original model can overwrite each other since each call starts from the same snapshot.

## Proposed Solution: Typed Update Builder

Generate a per-model builder that wraps SeaORM's `ActiveModel`:

```rust
// Before — counting Nones
item.update(None, None, None, None, Some(Some(json!(["glutine"]))), None, None, Some(true)).await?;

// After — named fields, only set what changes
item.update()
    .is_featured(true)
    .allergens(json!(["glutine"]))
    .save()
    .await?;
```

### Nullable field handling

For nullable columns, generate two methods:

```rust
// Set to a value
builder.allergens(json!(["glutine"]))  // sets to Some(value)

// Set to NULL
builder.clear_allergens()              // sets to None
```

This eliminates `Option<Option<T>>` from the public API entirely.

### Implementation sketch

```rust
// Auto-generated per model (macro or derive)
pub struct ItemUpdate {
    active: ActiveModel,
}

impl Item {
    pub fn update(&self) -> ItemUpdate {
        ItemUpdate {
            active: self.clone().into(),
        }
    }
}

impl ItemUpdate {
    pub fn slug(mut self, val: impl Into<String>) -> Self {
        self.active.slug = Set(val.into());
        self
    }

    pub fn price(mut self, val: Decimal) -> Self {
        self.active.price = Set(val);
        self
    }

    pub fn allergens(mut self, val: Json) -> Self {
        self.active.allergens = Set(Some(val));
        self
    }

    pub fn clear_allergens(mut self) -> Self {
        self.active.allergens = Set(None);
        self
    }

    pub fn is_featured(mut self, val: bool) -> Self {
        self.active.is_featured = Set(val);
        self
    }

    pub async fn save(self) -> Result<Item, FrameworkError> {
        Entity::update_one(self.active).await
    }
}
```

### Derive macro approach

```rust
#[derive(Model, UpdateBuilder)]  // generates ItemUpdate automatically
pub struct Item { ... }
```

The macro reads the entity fields and generates:
- One setter per non-PK column (typed, no Option wrapping)
- `clear_*` methods for nullable columns
- `save()` to execute

## Benefits

| Aspect | Positional | Builder |
|--------|-----------|---------|
| Add new column | Breaks all callers | Zero impact |
| Wrong field position | Silent bug | Compile error (named) |
| Readability | `None, None, None, None, Some(true)` | `.is_featured(true)` |
| Nullable semantics | `Option<Option<T>>` | `.set()` / `.clear()` |
| IDE autocomplete | Must check signature | Shows available fields |

## Migration path

1. Add `UpdateBuilder` derive alongside existing `update()` methods
2. Deprecate positional `update()` with a compiler warning
3. Remove in next major version

No breaking change if introduced as an additive derive.
