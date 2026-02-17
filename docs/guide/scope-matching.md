# Scope Matching

MTChat uses a two-level scope system to determine which users can discover and join a dialog. This controls the "Available" chat list in the SDK.

## Concept

When your backend creates a dialog via the Management API, it can attach **access scopes** that define who should be able to see and join the chat. The user's scope (provided in the SDK config) is matched against the dialog's scopes to determine visibility.

## Matching Algorithm

A dialog is visible to a user when **all three conditions** are met:

1. **Tenant matches** -- `user.tenant_uid == scope.tenant_uid`
2. **Level 1 intersects** -- at least one value in `user.scope_level1` matches a value in `scope.scope_level1`
3. **Level 2 intersects** -- at least one value in `user.scope_level2` matches a value in `scope.scope_level2`

**Logic**: `tenant AND (ANY scope_level1) AND (ANY scope_level2)`

## Example

### Dialog scope (set by Management API):

```json
{
  "tenant_uid": "acme-corp",
  "scope_level1": ["logistics", "sales"],
  "scope_level2": ["manager", "admin"]
}
```

### User A (matches):

```json
{
  "tenant_uid": "acme-corp",
  "scope_level1": ["logistics"],
  "scope_level2": ["manager"]
}
```

```
✓ tenant_uid matches: "acme-corp" == "acme-corp"
✓ scope_level1 intersects: "logistics" ∈ ["logistics", "sales"]
✓ scope_level2 intersects: "manager" ∈ ["manager", "admin"]
→ Result: VISIBLE (user can join)
```

### User B (does not match):

```json
{
  "tenant_uid": "acme-corp",
  "scope_level1": ["hr"],
  "scope_level2": ["manager"]
}
```

```
✓ tenant_uid matches
✗ scope_level1 does NOT intersect: "hr" ∉ ["logistics", "sales"]
→ Result: NOT VISIBLE
```

### User C (wrong tenant):

```json
{
  "tenant_uid": "other-company",
  "scope_level1": ["logistics"],
  "scope_level2": ["admin"]
}
```

```
✗ tenant_uid does NOT match: "other-company" ≠ "acme-corp"
→ Result: NOT VISIBLE
```

## Empty Scope Arrays

If a scope level array is **empty** on the dialog side, it matches **any** value from the user:

```json
{
  "tenant_uid": "acme-corp",
  "scope_level1": [],
  "scope_level2": ["admin"]
}
```

This scope matches all users in `acme-corp` tenant who have `admin` in their `scope_level2`, regardless of their `scope_level1` values.

## Multiple Scopes

A dialog can have **multiple access scopes**. A user is a potential participant if they match **any one** of the scopes:

```json
{
  "access_scopes": [
    {
      "tenant_uid": "acme-corp",
      "scope_level1": ["logistics"],
      "scope_level2": ["manager"]
    },
    {
      "tenant_uid": "partner-inc",
      "scope_level1": ["operations"],
      "scope_level2": ["driver"]
    }
  ]
}
```

This dialog is visible to logistics managers at Acme Corp **and** operations drivers at Partner Inc.

## Practical Use Cases

### Departments + Roles

```
scope_level1 = departments (logistics, sales, hr, finance)
scope_level2 = roles (admin, manager, viewer, driver)
```

### Regions + Permissions

```
scope_level1 = regions (north, south, east, west)
scope_level2 = permissions (read, write, approve)
```

### Teams + Seniority

```
scope_level1 = teams (team_a, team_b, team_c)
scope_level2 = levels (junior, senior, lead)
```

## SDK Configuration

Set the user's scope in the SDK config:

```typescript
const config: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  userId: user.id,
  scopeConfig: {
    tenant_uid: user.tenantId,
    scope_level1: user.departments,   // string[]
    scope_level2: user.permissions,   // string[]
  },
  userProfile: {
    displayName: user.name,
    company: user.company,
  },
}
```

The SDK passes scope parameters to the Chat API when fetching available dialogs:

```
GET /api/v1/dialogs?type=available
  &tenant_uid=acme-corp
  &scope_level1=logistics
  &scope_level2=manager
```

## Flow: From Creation to Join

1. Your backend calls Management API to create a dialog with access scopes
2. A user opens the SDK -- the "Available" tab shows dialogs matching their scope
3. The user clicks "Join" and provides their display name
4. The user becomes a direct participant and sees the dialog in "My Chats"
5. The user can now send/receive messages and get notifications
