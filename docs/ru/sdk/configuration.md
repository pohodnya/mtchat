# Конфигурация SDK

## MTChatConfig

```typescript
interface MTChatConfig {
  /** URL API (напр., 'https://chat.example.com') */
  baseUrl: string

  /** URL WebSocket (авто-определяется из baseUrl) */
  wsUrl?: string

  /** UUID текущего пользователя */
  userId: string

  /** Scope-конфигурация для контроля доступа */
  scopeConfig: ScopeConfig

  /** Профиль пользователя */
  userProfile: UserProfile

  /** Язык UI: 'ru' | 'en' | 'zh' (по умолчанию: 'ru') */
  locale?: Locale

  /** Авто-переподключение (по умолчанию: true) */
  reconnect?: boolean

  /** Интервал переподключения в мс (по умолчанию: 3000) */
  reconnectInterval?: number

  /** Интервал heartbeat в мс (по умолчанию: 30000) */
  heartbeatInterval?: number
}
```

## ScopeConfig

Определяет scope пользователя для отображения доступных диалогов.

```typescript
interface ScopeConfig {
  /** Нулевой уровень scope (напр., тенанты/организации) */
  scopeLevel0: string[]

  /** Первый уровень scope (напр., отделы) */
  scopeLevel1: string[]

  /** Второй уровень scope (напр., роли/права) */
  scopeLevel2: string[]
}
```

Подробнее: [Scope-правила](../guide/scope-matching.md)

## UserProfile

Отображается в чате при присоединении пользователя.

```typescript
interface UserProfile {
  displayName: string   // Отображаемое имя
  company: string       // Компания
  email?: string        // Email (опционально)
  phone?: string        // Телефон (опционально)
}
```

## Интернационализация

| Locale | Язык |
|--------|------|
| `ru` | Русский (по умолчанию) |
| `en` | Английский |
| `zh` | Китайский |

```vue
<MTChat :config="{ ...config, locale: 'ru' }" />
```

Смена языка реактивна -- компонент не перемонтируется, состояние чата сохраняется.

## Полный пример

```typescript
const config: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  userId: user.id,
  scopeConfig: {
    scopeLevel0: [user.tenantId],
    scopeLevel1: user.departments,
    scopeLevel2: user.permissions,
  },
  userProfile: {
    displayName: user.fullName,
    company: user.companyName,
    email: user.email,
    phone: user.phone,
  },
  locale: 'ru',
  reconnect: true,
  heartbeatInterval: 30000,
}
```
