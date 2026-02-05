# Plan 007: i18n Implementation for MTChat

## Status: ✅ COMPLETED (2025-02-05)

## Overview

Implement internationalization (i18n) in MTChat Vue SDK with support for Russian (default), English, and Chinese languages. Add language selector to the demo app.

## Text Strings Inventory

**Total: ~65 strings across components**

| Component | Strings | Examples |
|-----------|---------|----------|
| MTChat.vue | ~30 | tabs, buttons, statuses, date dividers, empty states |
| ChatInfoPanel.vue | ~8 | title, object types, participant labels |
| JoinDialog.vue | ~10 | form labels, buttons, template strings |
| FileViewer.vue | ~15 | controls, file type labels, loading states |
| PDFViewer.vue | ~8 | page labels, zoom controls |

## Architecture

### Lightweight i18n (no external dependencies)

```
mtchat-vue/src/
├── i18n/
│   ├── index.ts           # Export all
│   ├── translations.ts    # All translations { ru, en, zh }
│   └── useI18n.ts         # Composable with t(), plural(), formatDate()
└── types/index.ts         # Add Locale type to MTChatConfig
```

### Locale Prop Flow

```
MTChatConfig.locale → provide(I18N_KEY) → useI18n() in child components
```

## Implementation Steps

### Step 1: Create translations file

**File:** `mtchat-vue/src/i18n/translations.ts`

```typescript
export type Locale = 'ru' | 'en' | 'zh'

export const translations: Record<Locale, TranslationStrings> = {
  ru: {
    tabs: {
      myChats: 'Мои чаты',
      available: 'Доступные',
    },
    status: {
      connected: 'Подключено',
      disconnected: 'Отключено',
    },
    buttons: {
      join: 'Присоединиться',
      send: 'Отправить',
      cancel: 'Отмена',
      leaveChat: 'Покинуть чат',
      info: 'Информация',
    },
    chat: {
      participants: '{count} участников',
      canJoin: 'Можно присоединиться',
      noActiveChats: 'Нет активных чатов',
      noAvailableChats: 'Нет доступных чатов',
      noMessages: 'Нет сообщений',
      selectChat: 'Выберите чат для начала переписки',
      noChatForObject: 'Нет чата для этого объекта',
      newMessages: 'Новые сообщения',
      messageDeleted: 'Сообщение удалено',
      joinToSend: 'Присоединитесь к чату, чтобы отправлять сообщения',
    },
    input: {
      placeholder: 'Введите сообщение...',
      attachFiles: 'Прикрепить файлы',
    },
    tooltips: {
      chatInfo: 'Информация о чате',
      menu: 'Меню',
      reply: 'Ответить',
      scrollDown: 'Вниз',
      close: 'Закрыть',
    },
    dates: {
      today: 'Сегодня',
      yesterday: 'Вчера',
    },
    user: {
      you: 'Вы',
      youBadge: '(Вы)',
      creator: 'Создатель',
      anonymous: 'Сотрудник компании {company}',
    },
    infoPanel: {
      title: 'Информация о чате',
      participants: 'Участники',
      objectTypes: {
        tender: 'Тендер',
        order: 'Заказ',
        route: 'Рейс',
      },
    },
    joinDialog: {
      title: 'Присоединиться к чату',
      displayName: 'Отображаемое имя',
      company: 'Компания',
      showContacts: 'Показать контакты',
      joining: 'Присоединение...',
    },
    fileViewer: {
      loading: 'Загрузка...',
      loadingPdf: 'Загрузка PDF...',
      failedToLoad: 'Не удалось загрузить',
      page: 'стр.',
      zoomIn: 'Увеличить',
      zoomOut: 'Уменьшить',
      resetZoom: 'Сбросить масштаб',
      download: 'Скачать',
      previous: 'Предыдущий',
      next: 'Следующий',
      fileTypes: {
        word: 'Документ Word',
        excel: 'Таблица Excel',
        powerpoint: 'Презентация PowerPoint',
        zip: 'ZIP архив',
        rar: 'RAR архив',
        text: 'Текстовый файл',
        csv: 'CSV файл',
        json: 'JSON файл',
        xml: 'XML файл',
        video: 'Видео',
        audio: 'Аудио',
        file: 'Файл',
      },
    },
  },
  en: { /* English translations */ },
  zh: { /* Chinese translations */ },
}
```

### Step 2: Create useI18n composable

**File:** `mtchat-vue/src/i18n/useI18n.ts`

```typescript
import { computed, inject, provide, ref, type InjectionKey, type Ref } from 'vue'
import { translations, type Locale } from './translations'

export const I18N_KEY: InjectionKey<Ref<Locale>> = Symbol('mtchat-i18n')

export function provideI18n(locale: Locale = 'ru') {
  const localeRef = ref(locale)
  provide(I18N_KEY, localeRef)
  return localeRef
}

export function useI18n() {
  const locale = inject(I18N_KEY, ref('ru' as Locale))
  const t = computed(() => translations[locale.value])

  // Template interpolation: t('user.anonymous', { company: 'Acme' })
  function tt(key: string, params?: Record<string, string | number>): string {
    const keys = key.split('.')
    let value: any = t.value
    for (const k of keys) {
      value = value?.[k]
    }
    if (typeof value !== 'string') return key
    if (!params) return value
    return value.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? ''))
  }

  // Date formatting with locale
  function formatDate(date: Date): string {
    const now = new Date()
    if (isSameDay(date, now)) return t.value.dates.today
    const yesterday = new Date(now)
    yesterday.setDate(yesterday.getDate() - 1)
    if (isSameDay(date, yesterday)) return t.value.dates.yesterday

    return new Intl.DateTimeFormat(locale.value, {
      day: 'numeric',
      month: 'long',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
    }).format(date)
  }

  return { t, tt, formatDate, locale }
}
```

### Step 3: Update MTChatConfig

**File:** `mtchat-vue/src/types/index.ts`

```typescript
export type Locale = 'ru' | 'en' | 'zh'

export interface MTChatConfig {
  baseUrl: string
  token: string
  userId: string
  scopeConfig: ScopeConfig
  userProfile: UserProfile
  locale?: Locale  // Add this, default 'ru'
}
```

### Step 4: Update MTChat.vue

```typescript
// Setup
import { provideI18n, useI18n } from '../i18n'

const props = defineProps<{ config: MTChatConfig, /* ... */ }>()

// Provide locale to all child components
provideI18n(props.config.locale ?? 'ru')
const { t, tt, formatDate } = useI18n()
```

```vue
<!-- Template updates -->
<button>{{ t.tabs.myChats }}</button>
<span>{{ tt('chat.participants', { count: dialog.participants_count }) }}</span>
<div>{{ formatDate(new Date(message.sent_at)) }}</div>
```

### Step 5: Update child components

Update ChatInfoPanel.vue, JoinDialog.vue, FileViewer.vue with `useI18n()`.

### Step 6: Demo app - Add locale to settings

**File:** `mtchat-example/src/types/index.ts`

```typescript
export interface AppSettings {
  adminToken: string
  apiBaseUrl: string
  theme: 'light' | 'dark'
  locale: 'ru' | 'en' | 'zh'  // Add
}

export const DEFAULT_SETTINGS: AppSettings = {
  adminToken: '',
  apiBaseUrl: window.location.origin,
  theme: 'light',
  locale: 'ru',  // Russian default
}
```

### Step 7: Demo app - Add language selector

**File:** `mtchat-example/src/components/TMSLayout.vue`

Add next to theme toggle:

```vue
<button class="demo-nav-link" @click="cycleLocale">
  <i class="pi pi-globe" />
  {{ localeLabels[settings.locale] }}
</button>
```

```typescript
const localeLabels = { ru: 'Русский', en: 'English', zh: '中文' }
const locales: Locale[] = ['ru', 'en', 'zh']

function cycleLocale() {
  const idx = locales.indexOf(settings.value.locale)
  updateSettings({ locale: locales[(idx + 1) % locales.length] })
}
```

### Step 8: Pass locale to MTChat in demo pages

Update ChatPage.vue and InlinePage.vue:

```vue
<MTChat :config="{ ...config, locale: settings.locale }" />
```

## Files to Modify

### New files:
- `mtchat-vue/src/i18n/index.ts`
- `mtchat-vue/src/i18n/translations.ts`
- `mtchat-vue/src/i18n/useI18n.ts`

### Modified files:
- `mtchat-vue/src/types/index.ts` - Add Locale type
- `mtchat-vue/src/components/MTChat.vue` - Use i18n
- `mtchat-vue/src/components/chat/ChatInfoPanel.vue` - Use i18n
- `mtchat-vue/src/components/chat/JoinDialog.vue` - Use i18n
- `mtchat-vue/src/components/chat/FileViewer.vue` - Use i18n
- `mtchat-vue/src/components/chat/PDFViewer.vue` - Use i18n
- `mtchat-example/src/types/index.ts` - Add locale to AppSettings
- `mtchat-example/src/components/TMSLayout.vue` - Add language selector
- `mtchat-example/src/pages/ChatPage.vue` - Pass locale
- `mtchat-example/src/pages/InlinePage.vue` - Pass locale

## Verification

1. Build SDK: `cd mtchat-vue && npm run build`
2. Build demo: `cd mtchat-example && npm run build`
3. Run demo: `npm run dev`
4. Test language switching:
   - Click language toggle in sidebar
   - Verify all UI text changes in MTChat component
   - Verify date formatting changes
   - Verify join dialog text changes
   - Verify info panel text changes
5. Test persistence: refresh page, language should persist
6. Test each language: ru, en, zh
