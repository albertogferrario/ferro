# Examples

This appendix provides worked examples for all 7 standard intents. Each example shows a representative `ServiceDef` JSON, the expected primary `IntentScore`, and the structural signals that drive derivation.

Examples are abbreviated to highlight the distinctive structural elements. For a complete `ServiceDef` JSON example, see the [Process / Order Management](#process--order-management) section.

## Browse -- Product Catalog

A service with many fields annotated with `Category` and `EntityName`, multiple `OneToMany` relationships, and few writable fields signals Browse.

**ServiceDef (abbreviated):**

```json
{
  "name": "product",
  "display_name": "Product Catalog",
  "fields": [
    { "name": "id", "data_type": "integer", "meaning": "identifier", "required": true, "readable": true, "writable": false },
    { "name": "name", "data_type": "string", "meaning": "entity_name", "required": true, "readable": true, "writable": false },
    { "name": "category", "data_type": "string", "meaning": "category", "required": true, "readable": true, "writable": false },
    { "name": "subcategory", "data_type": "string", "meaning": "category", "required": false, "readable": true, "writable": false },
    { "name": "price", "data_type": "float", "meaning": "money", "required": true, "readable": true, "writable": false },
    { "name": "image", "data_type": "string", "meaning": "image_url", "required": false, "readable": true, "writable": false }
  ],
  "relationships": [
    { "name": "variants", "target": "product_variant", "cardinality": "one_to_many", "navigation": "nested" },
    { "name": "reviews", "target": "review", "cardinality": "one_to_many", "navigation": "tab" }
  ]
}
```

**Expected primary IntentScore:**

```json
{
  "intent": "browse",
  "confidence": 0.9,
  "matching_signals": [
    "category_field",
    "entity_name",
    "one_to_many_relationships",
    "multiple_relationships",
    "baseline"
  ]
}
```

**Signal analysis:** Multiple `Category` fields and `EntityName` trigger the field meaning analyzer's Browse signals. Two `OneToMany` relationships contribute strong Browse weight from the relationship analyzer. Low writable ratio reinforces the browsing pattern.

## Focus -- User Profile

A service with `EntityName`, `Email`, `Phone`, `ImageUrl`, readable fields, and few relationships signals Focus.

**ServiceDef (abbreviated):**

```json
{
  "name": "user_profile",
  "display_name": "User Profile",
  "fields": [
    { "name": "id", "data_type": "integer", "meaning": "identifier", "required": true, "readable": true, "writable": false },
    { "name": "display_name", "data_type": "string", "meaning": "entity_name", "required": true, "readable": true, "writable": true },
    { "name": "email", "data_type": "string", "meaning": "email", "required": true, "readable": true, "writable": true },
    { "name": "phone", "data_type": "string", "meaning": "phone", "required": false, "readable": true, "writable": true },
    { "name": "avatar", "data_type": "string", "meaning": "image_url", "required": false, "readable": true, "writable": true },
    { "name": "bio", "data_type": "string", "meaning": "free_text", "required": false, "readable": true, "writable": true },
    { "name": "website", "data_type": "string", "meaning": "url", "required": false, "readable": true, "writable": true }
  ]
}
```

**Expected primary IntentScore:**

```json
{
  "intent": "focus",
  "confidence": 0.85,
  "matching_signals": [
    "entity_name",
    "image_url_fields",
    "url_fields",
    "free_text_fields",
    "more_readable_than_writable",
    "baseline"
  ]
}
```

**Signal analysis:** `EntityName` combined with `ImageUrl`, `Url`, and `FreeText` produce Focus signals from the field meaning analyzer. No state machine, no `OneToMany` relationships, and a balanced read/write ratio reinforce the single-entity view pattern.

## Collect -- Feedback Form

A service with a majority of writable fields, `FreeText` fields, and no state machine signals Collect.

**ServiceDef (abbreviated):**

```json
{
  "name": "feedback",
  "display_name": "Feedback Form",
  "fields": [
    { "name": "id", "data_type": "integer", "meaning": "identifier", "required": true, "readable": true, "writable": false },
    { "name": "subject", "data_type": "string", "meaning": "entity_name", "required": true, "readable": false, "writable": true },
    { "name": "message", "data_type": "string", "meaning": "free_text", "required": true, "readable": false, "writable": true },
    { "name": "rating", "data_type": "integer", "meaning": "quantity", "required": true, "readable": false, "writable": true },
    { "name": "email", "data_type": "string", "meaning": "email", "required": false, "readable": false, "writable": true },
    { "name": "category", "data_type": "string", "meaning": "category", "required": false, "readable": false, "writable": true }
  ]
}
```

**Expected primary IntentScore:**

```json
{
  "intent": "collect",
  "confidence": 0.75,
  "matching_signals": [
    "high_writable_ratio",
    "write_only_fields"
  ]
}
```

**Signal analysis:** Write-only fields (writable but not readable) and a high overall writable ratio (>50%) trigger the writability analyzer's Collect signals. No state machine prevents Process/Track from competing.

## Process -- Order Management

A service with a state machine containing guards, transition triggers, and branching states signals Process. This example shows a complete `ServiceDef` JSON.

**ServiceDef (complete):**

```json
{
  "name": "order",
  "display_name": "Order",
  "description": "Manages customer orders through a fulfillment lifecycle",
  "fields": [
    {
      "name": "id",
      "data_type": "integer",
      "meaning": "identifier",
      "required": true,
      "is_list": false,
      "readable": true,
      "writable": false
    },
    {
      "name": "total",
      "data_type": "float",
      "meaning": "money",
      "required": true,
      "is_list": false,
      "readable": true,
      "writable": true
    },
    {
      "name": "status",
      "data_type": "string",
      "meaning": "status",
      "required": true,
      "is_list": false,
      "readable": true,
      "writable": false
    },
    {
      "name": "notes",
      "data_type": "string",
      "meaning": "free_text",
      "required": false,
      "is_list": false,
      "readable": true,
      "writable": true
    }
  ],
  "actions": [
    {
      "name": "submit",
      "display_name": "Submit Order",
      "inputs": [
        {
          "name": "notes",
          "data_type": "string",
          "meaning": "free_text",
          "required": false
        }
      ],
      "preconditions": ["has_items"],
      "transition_trigger": "submit"
    },
    {
      "name": "cancel",
      "display_name": "Cancel Order",
      "inputs": [],
      "preconditions": [],
      "transition_trigger": "cancel"
    }
  ],
  "guards": [
    {
      "name": "has_items",
      "display_name": "Has Items",
      "description": "Order must contain at least one item"
    },
    {
      "name": "payment_verified",
      "display_name": "Payment Verified",
      "description": "Payment has been successfully processed"
    }
  ],
  "relationships": [
    {
      "name": "items",
      "target": "order_item",
      "cardinality": "one_to_many",
      "navigation": "nested"
    },
    {
      "name": "customer",
      "target": "customer",
      "cardinality": "many_to_one",
      "navigation": "link",
      "foreign_key": "customer_id"
    }
  ],
  "state_machine": {
    "name": "order_lifecycle",
    "initial_state": "draft",
    "states": [
      { "name": "draft", "display_name": "Draft" },
      { "name": "submitted", "display_name": "Submitted" },
      { "name": "processing", "display_name": "Processing" },
      { "name": "completed", "display_name": "Completed", "is_final": true },
      { "name": "cancelled", "display_name": "Cancelled", "is_final": true }
    ],
    "transitions": [
      { "from": "draft", "event": "submit", "to": "submitted", "guard": "has_items" },
      { "from": "submitted", "event": "approve", "to": "processing", "guard": "payment_verified" },
      { "from": "processing", "event": "complete", "to": "completed" },
      { "from": "draft", "event": "cancel", "to": "cancelled" },
      { "from": "submitted", "event": "cancel", "to": "cancelled" }
    ]
  }
}
```

**Expected primary IntentScore:**

```json
{
  "intent": "process",
  "confidence": 0.95,
  "matching_signals": [
    "guarded_transitions",
    "transition_triggers",
    "branching_states",
    "workflow_states"
  ]
}
```

**Signal analysis:** The state machine analyzer produces the dominant signal. Guard density (2 guards across 5 transitions), branching states (both `draft` and `submitted` have multiple outgoing transitions), and transition triggers on actions all amplify the Process signal. The action analyzer reinforces with transition trigger and precondition signals.

## Summarize -- Revenue Dashboard

A service with `Money`, `Percentage`, and `Quantity` fields that are mostly non-writable signals Summarize.

**ServiceDef (abbreviated):**

```json
{
  "name": "revenue_dashboard",
  "display_name": "Revenue Dashboard",
  "fields": [
    { "name": "id", "data_type": "integer", "meaning": "identifier", "required": true, "readable": true, "writable": false },
    { "name": "total_revenue", "data_type": "float", "meaning": "money", "required": true, "readable": true, "writable": false },
    { "name": "monthly_revenue", "data_type": "float", "meaning": "money", "required": true, "readable": true, "writable": false },
    { "name": "growth_rate", "data_type": "float", "meaning": "percentage", "required": true, "readable": true, "writable": false },
    { "name": "order_count", "data_type": "integer", "meaning": "quantity", "required": true, "readable": true, "writable": false },
    { "name": "avg_order_value", "data_type": "float", "meaning": "money", "required": true, "readable": true, "writable": false }
  ]
}
```

**Expected primary IntentScore:**

```json
{
  "intent": "summarize",
  "confidence": 0.85,
  "matching_signals": [
    "money_fields",
    "percentage_fields",
    "quantity_fields",
    "mostly_read_only"
  ]
}
```

**Signal analysis:** Multiple `Money` fields, a `Percentage` field, and a `Quantity` field drive the field meaning analyzer's Summarize signals. The writability analyzer contributes an additional Summarize signal from the >70% non-writable ratio.

## Analyze -- Sales Analytics

A service with `DateTime` fields co-occurring with numeric fields (`Money`, `Quantity`), sortable columns, and mixed readability signals Analyze.

**ServiceDef (abbreviated):**

```json
{
  "name": "sales_analytics",
  "display_name": "Sales Analytics",
  "fields": [
    { "name": "id", "data_type": "integer", "meaning": "identifier", "required": true, "readable": true, "writable": false },
    { "name": "sale_date", "data_type": "date_time", "meaning": "date_time", "required": true, "readable": true, "writable": true },
    { "name": "amount", "data_type": "float", "meaning": "money", "required": true, "readable": true, "writable": true },
    { "name": "units_sold", "data_type": "integer", "meaning": "quantity", "required": true, "readable": true, "writable": true },
    { "name": "region", "data_type": "string", "meaning": "category", "required": true, "readable": true, "writable": true },
    { "name": "product_name", "data_type": "string", "meaning": "entity_name", "required": true, "readable": true, "writable": true }
  ]
}
```

**Expected primary IntentScore:**

```json
{
  "intent": "analyze",
  "confidence": 0.6,
  "matching_signals": [
    "datetime_numeric_cooccurrence"
  ]
}
```

**Signal analysis:** The co-occurrence of `DateTime` and numeric fields (`Money`, `Quantity`) triggers the field meaning analyzer's Analyze signal. Mixed read/write fields prevent Summarize from dominating (which requires >70% non-writable). The `Category` field adds a secondary Browse signal, but Analyze wins on the DateTime-numeric pattern.

## Track -- Todo Tracker

A service with a `Status` field, a linear state machine (no branching, no guards), and `DateTime` fields signals Track.

**ServiceDef (abbreviated):**

```json
{
  "name": "todo",
  "display_name": "Todo",
  "fields": [
    { "name": "id", "data_type": "integer", "meaning": "identifier", "required": true, "readable": true, "writable": false },
    { "name": "title", "data_type": "string", "meaning": "entity_name", "required": true, "readable": true, "writable": true },
    { "name": "status", "data_type": "string", "meaning": "status", "required": true, "readable": true, "writable": false },
    { "name": "due_date", "data_type": "date_time", "meaning": "date_time", "required": false, "readable": true, "writable": true },
    { "name": "created_at", "data_type": "date_time", "meaning": "created_at", "required": true, "readable": true, "writable": false },
    { "name": "updated_at", "data_type": "date_time", "meaning": "updated_at", "required": true, "readable": true, "writable": false }
  ],
  "state_machine": {
    "name": "todo_lifecycle",
    "initial_state": "pending",
    "states": [
      { "name": "pending", "display_name": "Pending" },
      { "name": "in_progress", "display_name": "In Progress" },
      { "name": "done", "display_name": "Done", "is_final": true }
    ],
    "transitions": [
      { "from": "pending", "event": "start", "to": "in_progress" },
      { "from": "in_progress", "event": "complete", "to": "done" }
    ]
  }
}
```

**Expected primary IntentScore:**

```json
{
  "intent": "track",
  "confidence": 0.75,
  "matching_signals": [
    "status_field",
    "linear_states",
    "has_final_states",
    "unguarded_progression"
  ]
}
```

**Signal analysis:** The `Status` field triggers the field meaning analyzer's Track signal. The state machine analyzer detects a linear progression (each state has at most one outgoing transition), the presence of a final state, and the absence of guards -- all hallmarks of a tracking workflow rather than a complex process.
