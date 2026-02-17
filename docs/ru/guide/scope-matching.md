# Scope-сопоставление

MTChat использует двухуровневую систему scope для определения, какие пользователи могут обнаруживать и присоединяться к диалогам. Это управляет вкладкой "Доступные" в SDK.

## Концепция

Когда ваш бэкенд создает диалог через Management API, он может прикрепить **scope доступа**, определяющие, кто сможет видеть и присоединяться к чату. Scope пользователя (указанный в конфигурации SDK) сопоставляется со scope диалога для определения видимости.

## Алгоритм сопоставления

Диалог виден пользователю, когда **все три условия** выполнены:

1. **Тенант совпадает** -- `user.tenant_uid == scope.tenant_uid`
2. **Level 1 пересекается** -- хотя бы одно значение в `user.scope_level1` совпадает со значением в `scope.scope_level1`
3. **Level 2 пересекается** -- хотя бы одно значение в `user.scope_level2` совпадает со значением в `scope.scope_level2`

**Логика**: `tenant AND (ANY scope_level1) AND (ANY scope_level2)`

## Пример

### Scope диалога (заданный через Management API):

```json
{
  "tenant_uid": "acme-corp",
  "scope_level1": ["logistics", "sales"],
  "scope_level2": ["manager", "admin"]
}
```

### Пользователь А (совпадает):

```json
{
  "tenant_uid": "acme-corp",
  "scope_level1": ["logistics"],
  "scope_level2": ["manager"]
}
```

```
✓ tenant_uid совпадает: "acme-corp" == "acme-corp"
✓ scope_level1 пересекается: "logistics" ∈ ["logistics", "sales"]
✓ scope_level2 пересекается: "manager" ∈ ["manager", "admin"]
→ Результат: ВИДИМ (может присоединиться)
```

### Пользователь Б (не совпадает):

```json
{
  "tenant_uid": "acme-corp",
  "scope_level1": ["hr"],
  "scope_level2": ["manager"]
}
```

```
✓ tenant_uid совпадает
✗ scope_level1 НЕ пересекается: "hr" ∉ ["logistics", "sales"]
→ Результат: НЕ ВИДИМ
```

### Пользователь В (другой тенант):

```json
{
  "tenant_uid": "other-company",
  "scope_level1": ["logistics"],
  "scope_level2": ["admin"]
}
```

```
✗ tenant_uid НЕ совпадает: "other-company" ≠ "acme-corp"
→ Результат: НЕ ВИДИМ
```

## Пустые массивы scope

Если массив scope **пустой** на стороне диалога, он совпадает с **любым** значением пользователя:

```json
{
  "tenant_uid": "acme-corp",
  "scope_level1": [],
  "scope_level2": ["admin"]
}
```

Этот scope подходит всем пользователям тенанта `acme-corp`, у которых есть `admin` в `scope_level2`, независимо от их значений `scope_level1`.

## Множественные scope

Диалог может иметь **несколько scope доступа**. Пользователь является потенциальным участником, если он совпадает с **любым одним** из scope:

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

Этот диалог виден менеджерам логистики в Acme Corp **и** водителям отдела операций в Partner Inc.

## Практические сценарии

### Департаменты + Роли

```
scope_level1 = департаменты (logistics, sales, hr, finance)
scope_level2 = роли (admin, manager, viewer, driver)
```

### Регионы + Разрешения

```
scope_level1 = регионы (north, south, east, west)
scope_level2 = разрешения (read, write, approve)
```

### Команды + Уровень

```
scope_level1 = команды (team_a, team_b, team_c)
scope_level2 = уровни (junior, senior, lead)
```

## Конфигурация SDK

Задайте scope пользователя в конфигурации SDK:

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

## Путь: от создания до присоединения

1. Ваш бэкенд вызывает Management API для создания диалога со scope доступа
2. Пользователь открывает SDK -- вкладка "Доступные" показывает диалоги, совпадающие с его scope
3. Пользователь нажимает "Присоединиться" и указывает имя
4. Пользователь становится прямым участником и видит диалог в "Участвую"
5. Пользователь может отправлять/получать сообщения и получать уведомления
