# Интеграция PrimeVue

Пакет `@mtchat/vue-primevue` обеспечивает интеграцию с [PrimeVue](https://primevue.org/) 4.x. Он заменяет встроенные UI-примитивы MTChat (кнопки, диалоги, инпуты, табы) на PrimeVue-эквиваленты и маппит ваши токены темы PrimeVue.

## Установка

```bash
npm install @mtchat/vue @mtchat/vue-primevue primevue
```

### Зависимости

| Пакет | Версия |
|-------|--------|
| `vue` | `^3.4.0` |
| `primevue` | `^4.0.0` |
| `@mtchat/vue` | `^0.4.0` |

## Быстрый старт

Используйте `MTChatPrime` вместо `MTChat`:

```vue
<script setup lang="ts">
import { MTChatPrime, type MTChatConfig } from '@mtchat/vue-primevue'

const config: MTChatConfig = {
  baseUrl: 'https://chat.example.com',
  userId: currentUser.id,
  scopeConfig: {
    scopeLevel0: [currentUser.tenantId],
    scopeLevel1: currentUser.departments,
    scopeLevel2: currentUser.permissions,
  },
  userProfile: {
    displayName: currentUser.name,
    company: currentUser.company,
  },
}
</script>

<template>
  <div style="height: 600px;">
    <MTChatPrime :config="config" theme="light" />
  </div>
</template>
```

## Маппинг темы

PrimeVue-интеграция автоматически маппит CSS-переменные PrimeVue на переменные MTChat:

| MTChat | PrimeVue |
|--------|----------|
| `--mtchat-primary` | `--p-primary-color` |
| `--mtchat-bg` | `--p-surface-0` |
| `--mtchat-text` | `--p-text-color` |
| `--mtchat-border` | `--p-surface-border` |

`MTChatPrime` уже импортирует bundled theme mapping внутри себя. Если используете manual registry path, импортируйте `@mtchat/vue-primevue/theme/aura.css` вручную.

## Компоненты

PrimeVue-пакет заменяет следующие внутренние компоненты:

| MTChat компонент | PrimeVue аналог |
|------------------|-----------------|
| `MtButton` | `Button` |
| `MtDialog` | `Dialog` |
| `MtMenu` | `Menu` |
| `MtContextMenu` | `ContextMenu` |
| `MtInput` | `InputText` |
| `MtCheckbox` | `Checkbox` |
| `MtRadioButton` | `RadioButton` |
| `MtTabs` / `MtTab` | `Tabs` / `Tab` |
| `MtAccordion` / `MtAccordionPanel` | `Accordion` / `AccordionPanel` |

## Ручная настройка registry

```vue
<script setup lang="ts">
import { MTChat, provideRegistry } from '@mtchat/vue'
import { primevueRegistry } from '@mtchat/vue-primevue'
import '@mtchat/vue-primevue/theme/aura.css'

provideRegistry(primevueRegistry)
</script>

<template>
  <div style="height: 600px;">
    <MTChat :config="config" />
  </div>
</template>
```
