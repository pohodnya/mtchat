# Scope-сопоставление

MTChat использует трёхуровневую систему scope для определения, какие пользователи могут обнаруживать и присоединяться к диалогам. Это управляет вкладкой "Доступные" в SDK.

## Концепция

Когда ваш бэкенд создает диалог через Management API, он может прикрепить **scope доступа**, определяющие, кто сможет видеть и присоединяться к чату. Scope пользователя (указанный в конфигурации SDK) сопоставляется со scope диалога для определения видимости.

## Алгоритм сопоставления

Диалог виден пользователю, когда **все три условия** выполнены:

1. **Level 0 пересекается** -- хотя бы одно значение в `user.scope_level0` совпадает со значением в `scope.scope_level0`
2. **Level 1 пересекается** -- хотя бы одно значение в `user.scope_level1` совпадает со значением в `scope.scope_level1`
3. **Level 2 пересекается** -- хотя бы одно значение в `user.scope_level2` совпадает со значением в `scope.scope_level2`

**Логика**: `(ANY scope_level0) AND (ANY scope_level1) AND (ANY scope_level2)`

## Пример

### Scope диалога (заданный через Management API):

```json
{
  "scope_level0": ["acme-corp"],
  "scope_level1": ["logistics", "sales"],
  "scope_level2": ["manager", "admin"]
}
```

### Пользователь А (совпадает):

```json
{
  "scope_level0": ["acme-corp"],
  "scope_level1": ["logistics"],
  "scope_level2": ["manager"]
}
```

```
✓ scope_level0 пересекается: "acme-corp" ∈ ["acme-corp"]
✓ scope_level1 пересекается: "logistics" ∈ ["logistics", "sales"]
✓ scope_level2 пересекается: "manager" ∈ ["manager", "admin"]
→ Результат: ВИДИМ (может присоединиться)
```

### Пользователь Б (не совпадает):

```json
{
  "scope_level0": ["acme-corp"],
  "scope_level1": ["hr"],
  "scope_level2": ["manager"]
}
```

```
✓ scope_level0 пересекается
✗ scope_level1 НЕ пересекается: "hr" ∉ ["logistics", "sales"]
→ Результат: НЕ ВИДИМ
```

### Пользователь В (другой тенант):

```json
{
  "scope_level0": ["other-company"],
  "scope_level1": ["logistics"],
  "scope_level2": ["admin"]
}
```

```
✗ scope_level0 НЕ пересекается: "other-company" ∉ ["acme-corp"]
→ Результат: НЕ ВИДИМ
```

## Пустые массивы scope

Если массив scope **пустой** на стороне диалога, он совпадает с **любым** значением пользователя:

```json
{
  "scope_level0": ["acme-corp"],
  "scope_level1": [],
  "scope_level2": ["admin"]
}
```

Этот scope подходит всем пользователям с `acme-corp` в `scope_level0`, у которых есть `admin` в `scope_level2`, независимо от их значений `scope_level1`.

## Множественные scope

Диалог может иметь **несколько scope доступа**. Пользователь является потенциальным участником, если он совпадает с **любым одним** из scope:

```json
{
  "access_scopes": [
    {
      "scope_level0": ["acme-corp"],
      "scope_level1": ["logistics"],
      "scope_level2": ["manager"]
    },
    {
      "scope_level0": ["partner-inc"],
      "scope_level1": ["operations"],
      "scope_level2": ["driver"]
    }
  ]
}
```

Этот диалог виден менеджерам логистики в Acme Corp **и** водителям отдела операций в Partner Inc.

## Практические сценарии

### Тенанты + Департаменты + Роли

```
scope_level0 = тенанты (acme-corp, partner-inc)
scope_level1 = департаменты (logistics, sales, hr, finance)
scope_level2 = роли (admin, manager, viewer, driver)
```

### Организации + Регионы + Разрешения

```
scope_level0 = организации (org-1, org-2)
scope_level1 = регионы (north, south, east, west)
scope_level2 = разрешения (read, write, approve)
```

### Компании + Команды + Уровень

```
scope_level0 = компании (company-a, company-b)
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
    scopeLevel0: [user.tenantId],     // string[]
    scopeLevel1: user.departments,    // string[]
    scopeLevel2: user.permissions,    // string[]
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
