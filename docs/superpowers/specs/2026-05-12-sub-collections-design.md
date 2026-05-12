# Sub-Collections Design

**Date:** 2026-05-12  
**Status:** Approved

## Overview

Add one-level-deep parent/child hierarchy to collections. A parent collection is a named container that holds child collections — it never holds cards directly. Clicking a parent in the UI shows the aggregate of all its children's cards. Any existing collection can become a parent.

## Use Case

User has physical boxes (Box 1, Box 2) each containing labeled sub-collections (A1–A12, A13–A24). The parent ("Box 1") acts as a navigation shortcut to find which box to open, while child collections remain independently browsable.

## Data Model

### Migration 05

Add a nullable self-referential foreign key to the `collection` table:

```sql
ALTER TABLE collection ADD COLUMN parent TEXT REFERENCES collection(name) NULL;
```

- A collection with `parent IS NULL` and children referencing it = a parent container
- A collection with `parent IS NULL` and no children = a top-level leaf
- A collection with `parent IS NOT NULL` = a child collection
- Parenthood is implicit — no separate flag needed
- Nesting is one level deep; the schema supports extension to two levels later

### Rename Cascade

`rename_collection` updates `parent` on all children in the same transaction:

```sql
UPDATE collection SET parent = ?new WHERE parent = ?old
```

### Delete Parent

When a parent is deleted, the server accepts an optional `reparentChildrenTo: String | null`:
- `null` → children' `parent` set to NULL (become top-level)
- Some other parent name → children moved to that parent

The confirmation dialog lives in the frontend; the server handles whichever instruction it receives.

## Backend / API

### Updated Response Model

`GET /collections/list` adds `parent: string | null` to each collection:

```json
{ "id": "A1", "canRemove": true, "isProxy": false, "parent": "Box 1" }
```

The `Collection` struct in `collections_models.rs` and `models::Collection` both gain an `Option<String> parent` field.

### Updated Add Endpoint

`POST /collections/add` gains an optional `parent: string | null` field. When provided, the new collection is created as a child of the specified parent. Validation: parent must exist and must not itself have a parent.

### New Endpoint

`POST /collections/set-parent/{id}`  
Body: `{ "parent": "Box 1" }` or `{ "parent": null }`

Validation:
- Target parent must exist
- Target cannot be the collection itself
- Target cannot be a child of the collection being updated (no cycles)
- A parent collection cannot itself be assigned a parent (one level enforced)

### Updated Remove Endpoint

`POST /collections/remove/{id}` gains an optional `reparentChildrenTo: string | null` field alongside the existing `moveTo` (which moves cards). Both can be provided independently.

### Card Aggregation

`get_cards_for_collection_scope` gains a check: if the requested `collection_id` has children in the DB, aggregate cards across all children (same pattern as `__all__`). This makes:

- `/c/Box1/1` show all cards across A1–A12
- Searching within Box 1 search across all children
- Stats for Box 1 aggregate across children

The check is a single query: `SELECT COUNT(*) FROM collection WHERE parent = ?id`.

### Persistence Trait

New method:

```rust
fn set_collection_parent(
    &mut self,
    collection_id: &CollectionID,
    parent: Option<CollectionID>,
) -> impl Future<Output = eyre::Result<Collection>>;
```

`list_collection_details` returns `parent: Option<String>` on each `Collection`.

## Frontend

### Sidebar Layout

Collections from the API are grouped client-side before rendering:

1. Build a map of `parent → [children]`
2. Render top-level collections (those with `parent === null`) first
3. For each top-level collection that has children, render it as an expandable parent row
4. Children are indented below their parent, shown when expanded

**Default state:** parents are collapsed.

**Parent row behavior:** clicking the collection name both expands the child list *and* navigates to `/c/{parentName}/1` (which returns aggregated cards from all children).

### Sidebar Item Types

**Parent row** (has children):
- Chevron icon (▶ collapsed / ▼ expanded), toggles child list
- Collection name as nav link → navigates + expands
- On hover: `+` button (add child collection inline), `...` menu

**Child row** (has parent):
- Indented under parent
- Collection name as nav link
- On hover: `...` menu

**Top-level leaf** (no parent, no children):
- Collection name as nav link
- On hover: `...` menu

### `...` Menu Contents

Appears on hover for every collection row. Contents vary by type:

| Action | Parent | Child | Top-level leaf |
|---|---|---|---|
| Rename | ✓ | ✓ | ✓ |
| Delete | ✓ | ✓ | ✓ |
| Move into a parent | — | — | ✓ |
| Move to another parent | — | ✓ | — |
| Remove from parent | — | ✓ | — |

Rename and Delete work as they do today. Move actions open a modal with a dropdown of available parent collections.

### Add Child Collection

The `+` button on a parent row shows an inline input (same pattern as `AddCollectionForm`) that submits with the parent pre-filled. On submit: `POST /collections/add` with `{ id: "A1", parent: "Box 1" }`.

The existing `AddCollectionForm` at the bottom of the sidebar remains for adding top-level collections.

### Delete Parent Dialog

When the user deletes a parent that still has children, a confirmation dialog appears:

> "Box 1 has 3 child collections. What should happen to them?"
> - [ ] Move to another parent: [dropdown]
> - [ ] Leave as top-level collections
> [Cancel] [Delete]

### Searching Within a Parent

The existing collection search UI (`/c/{id}/search`) works automatically since the backend aggregates children's cards when the collection is a parent. No frontend changes needed for search.

## Constraints

- A parent cannot be assigned a parent (one level enforced in `set-parent` validation)
- A collection cannot be its own parent
- Cards cannot be added directly to a parent collection (validated server-side: `SELECT COUNT(*) FROM collection WHERE parent = ?id > 0` before accepting a card add; return 400 if true)
- The `Default` collection (can_remove=false) can be a child or a top-level leaf but cannot be a parent (it always holds cards directly)

## Out of Scope

- Two-level nesting (grandparent → parent → child): schema supports it but UI and validation intentionally limit to one level for now
- Drag-and-drop reordering of collections or children
- Bulk moving multiple collections to a parent at once
